use anyhow::Result;
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::{fs::File, rc::Rc, vec};

use crate::{
    api::{self, Action, ApiData},
    backup::MinecraftSaveCollection,
    globals::CONFIG_FILE,
};

slint::include_modules!();

pub fn run_ui() -> Result<()> {
    let app = Main::new()?;
    load_settings(&app)?;
    register(&app);
    app.run()?;
    Ok(())
}

fn load_settings(app: &Main) -> Result<()> {
    if let Ok(reader) = File::open(CONFIG_FILE.as_path()) {
        app.global::<Settings>()
            .set_settings(serde_json::from_reader(reader)?);
    } else {
        app.global::<Settings>().set_settings(Default::default());
    }

    Ok(())
}

fn ensure_vec_models(app: &Main) {
    let tasks = app.global::<Tasks>();
    let status = Rc::new(VecModel::from_iter(tasks.get_status().iter()));
    let names = Rc::new(VecModel::from_iter(tasks.get_name().iter()));
    let results = Rc::new(VecModel::from_iter(tasks.get_result().iter()));

    tasks.set_status(ModelRc::from(status.clone()));
    tasks.set_name(ModelRc::from(names.clone()));
    tasks.set_result(ModelRc::from(results.clone()));
}

fn register(app: &Main) {
    let app_weak = app.as_weak();
    ensure_vec_models(app);
    app.global::<Settings>().on_settings_save(move || {
        let app = app_weak.upgrade().unwrap();
        let writer = File::create(CONFIG_FILE.as_path());
        if let Err(err) = &writer {
            add_task_info(&app, "Save configuration", 1, &err.to_string());
        }
        if let Err(err) =
            serde_json::to_writer_pretty(writer.unwrap(), &app.global::<Settings>().get_settings())
        {
            add_task_info(&app, "Save configuration", 1, &err.to_string());
        }
        add_task_info(&app, "Save configuration", 0, "OK");
    });
    let app_weak = app.as_weak();
    app.global::<Backups>().on_run_backup(move |id, name, t| {
        let app = app_weak.upgrade().unwrap();
        spawn_task(
            &app,
            format!("Backup: {}", name),
            Action::Backup,
            vec![format!("{}", id), format!("{}", t)],
        );
    });
    let app_weak = app.as_weak();
    app.global::<Backups>().on_refresh(move || {
        let app = app_weak.upgrade().unwrap();
        let backups = app.global::<Backups>();
        {
            MinecraftSaveCollection::global().lock().unwrap().refresh();
        }
        let mut result: Vec<MinecraftSaveMeta> = Vec::new();
        for item in MinecraftSaveCollection::global()
            .lock()
            .unwrap()
            .saves
            .values()
        {
            let mut versions = vec![];
            if let Some(version) = &item.latest_version {
                let mut p = version;
                versions.push(MinecraftSaveVersionMeta {
                    r#type: unsafe { std::mem::transmute_copy(&p.version_type) },
                    description: p.description.to_shared_string(),
                });
                while let Some(version) = &p.prev {
                    p = version;
                    versions.push(MinecraftSaveVersionMeta {
                        r#type: unsafe { std::mem::transmute_copy(&version.version_type) },
                        description: version.description.to_shared_string(),
                    });
                }
            }
            result.push(MinecraftSaveMeta {
                id: item.id.to_shared_string(),
                name: item.name.to_shared_string(),
                versions: ModelRc::new(VecModel::from(versions)),
            });
        }
        backups.set_backups(ModelRc::new(VecModel::from(result)));
    });
    app.global::<Backups>().invoke_refresh();
}
fn add_task_info<S: AsRef<str>>(app: &Main, name: S, status: i32, result: S) {
    let tasks = app.global::<Tasks>();

    let names = tasks.get_name();
    let status_ = tasks.get_status();
    let results = tasks.get_result();
    let names: &VecModel<SharedString> = names.as_any().downcast_ref().unwrap();
    let status_: &VecModel<i32> = status_.as_any().downcast_ref().unwrap();
    let results: &VecModel<SharedString> = results.as_any().downcast_ref().unwrap();
    names.push(name.as_ref().to_shared_string());
    status_.push(status);
    results.push(result.as_ref().to_shared_string());

    tasks.set_index(tasks.get_index() + 1);
}
fn spawn_task(app: &Main, name: String, action: Action, payload: Vec<String>) {
    let app_weak = app.as_weak();
    let tasks = app.global::<Tasks>();

    let names = tasks.get_name();
    let status = tasks.get_status();
    let results = tasks.get_result();
    let names: &VecModel<SharedString> = names.as_any().downcast_ref().unwrap();
    let status: &VecModel<i32> = status.as_any().downcast_ref().unwrap();
    let results: &VecModel<SharedString> = results.as_any().downcast_ref().unwrap();

    let result = slint::spawn_local(async move {
        let result =
            tokio::spawn(async move { api::handle_request(ApiData { action, payload }).await })
                .await;
        slint::invoke_from_event_loop(move || {
            let app = app_weak.upgrade().unwrap();
            let tasks = app.global::<Tasks>();
            let status = tasks.get_status();
            let results = tasks.get_result();
            let status: &VecModel<i32> = status.as_any().downcast_ref().unwrap();
            let results: &VecModel<SharedString> = results.as_any().downcast_ref().unwrap();

            status.remove(status.iter().len() - 1);
            if let Err(err) = result {
                status.push(1);
                results.push(slint::format!("{}", err));
            } else {
                status.push(0);
                results.push("OK".to_shared_string());
            }
        })
        .unwrap();
    });
    names.push(name.to_shared_string());
    if let Err(err) = result {
        status.push(1);
        results.push(slint::format!("Failed to spawn task: {}", err));
    } else {
        status.push(255);
    }
    tasks.set_index(tasks.get_index() + 1);
}
