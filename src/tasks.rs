use anyhow::{Result, anyhow};
use rustic_core::{SnapshotOptions, repofile::SnapshotFile};
use std::{
    sync::Mutex,
    thread::{self, JoinHandle},
};

use crate::{backup::AppState, tasks, utils::report_err};

static TASKS: Mutex<Vec<Task>> = Mutex::new(vec![]);

pub fn spawn(name: String, task_info: TaskInfo) {
    debug!(
        "task_spawn(id={}, name={})",
        TASKS.lock().expect("Unable to lock TASKS").len(),
        name
    );
    let worker = match task_info.clone() {
        TaskInfo::Backup { for_id, options } => thread::spawn(move || {
            if let Some(id) = for_id {
                let mut instance = AppState::instance();
                let save = instance.saves.get_mut(&id).ok_or(anyhow!("Invaild id"))?;
                save.run_backup(options)?;
                drop(instance);
            } else {
                let mut instance = AppState::instance();
                for save in instance.saves.values_mut() {
                    save.run_backup(options.clone())?;
                }
                drop(instance);
            }
            AppState::instance().save()
        }),
        TaskInfo::Recover { id, snapshot } => thread::spawn(move || {
            let instance = AppState::instance();
            instance.saves[&id].recover(snapshot)?;
            Ok(())
        }),
    };
    TASKS.lock().expect("Unable to lock TASKS").push(Task {
        name,
        info: task_info,
        progress: 0,
        worker: Some(worker),
    });
}

pub fn wait_all() {
    let mut tasks = TASKS.lock().unwrap();
    let mut id: usize = 0;
    while let Some(task) = tasks.pop() {
        info!("waiting #{} to finish", id);
        if let Err(err) = task.worker.unwrap().join() {
            error!("#{}: {:?}", id, err);
        }
        info!("#{} finished", id);
        id += 1;
    }
}

pub struct Task {
    name: String,
    info: TaskInfo,
    progress: u8,
    worker: Option<JoinHandle<Result<()>>>,
}

#[derive(Debug, Clone)]
pub enum TaskInfo {
    Backup {
        for_id: Option<String>,
        options: SnapshotOptions,
    },
    Recover {
        id: String,
        snapshot: SnapshotFile,
    },
}
