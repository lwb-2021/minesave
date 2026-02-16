use gtk4::{
    Box, Button, Label, TextView,
    prelude::{BoxExt, ButtonExt, EditableExt, EntryExt, TextBufferExt, TextViewExt},
};
use native_dialog::DialogBuilder;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::io::Write;

use crate::{
    settings::Settings,
    ui::{
        pages::build_wrapper,
        utils::{title, with_label},
    },
    utils::report_err,
};

pub fn settings() -> Box {
    let wrapper = build_wrapper();
    let save_button = Button::builder()
        .halign(gtk4::Align::End)
        .label(&t!("pages.settings.save").to_string())
        .build();

    let (b1, compression_level_input) = with_label::text_input(
        t!("pages.settings.compression-level"),
        Settings::instance().compression_level.to_string(),
    );
    let (b2, daemon_backup_duration_input_box) = with_label::text_input(
        t!("pages.settings.daemon-backup-duration"),
        Settings::instance().daemon_backup_duration.to_string(),
    );
    let (b3, pass_input_box) = with_label::text_input(
        t!("pages.settings.password"),
        Settings::instance().password.clone().unwrap_or_default(),
    );

    pass_input_box.set_visibility(false);
    let (b4, pass_cmd_input_box) = with_label::text_input(
        t!("pages.settings.password-command"),
        Settings::instance()
            .password_cmd
            .clone()
            .unwrap_or_default(),
    );

    let (b6, sync_switch) =
        with_label::switch(t!("pages.settings.sync"), Settings::instance().sync);

    let scan_root_input: TextView = TextView::builder().build();
    let scan_root_input_buffer0 = scan_root_input.buffer();
    let scan_root_input_buffer1 = scan_root_input.buffer();
    scan_root_input_buffer0.set_text(
        &Settings::instance()
            .scan_root
            .iter()
            .map(|x| x.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join("\n"),
    );

    let add_scan_root_button = Button::builder()
        .halign(gtk4::Align::Start)
        .label(&t!("pages.settings.add-scan-root").to_string())
        .build();
    add_scan_root_button.connect_clicked(move |_| {
        if let Ok(result) = native_dialog::FileDialogBuilder::default()
            .open_single_dir()
            .show()
            .inspect_err(report_err("Failed to open dialog"))
        {
            match result {
                Some(path) => {
                    Settings::instance().scan_root.push(path);
                    scan_root_input_buffer1.set_text(
                        &Settings::instance()
                            .scan_root
                            .iter()
                            .map(|x| x.to_string_lossy().to_string())
                            .collect::<Vec<String>>()
                            .join("\n"),
                    );
                    Settings::instance().save();
                }
                None => native_dialog::MessageDialogBuilder::default()
                    .set_text(t!("messages.action-cancelled"))
                    .alert()
                    .show()
                    .inspect_err(report_err("Failed to open dialog"))
                    .unwrap_or_default(),
            }
        }
    });

    let enable_auto_backup_button = Button::builder()
        .halign(gtk4::Align::Start)
        .label(&t!("pages.settings.enable-auto-backup").to_string())
        .build();

    #[cfg(target_os = "windows")]
    enable_auto_backup_button.connect_clicked(|_| {
        match std::fs::File::create(
            dirs::data_dir()
                .unwrap()
                .join("Microsoft")
                .join("Windows")
                .join("StartMenu")
                .join("Programs")
                .join("Startup")
                .join("MineSaveAutoBackup.bat"),
        ) {
            Ok(mut file) => {
                if let Err(err) =
                    write!(&mut file, "@start /b {}", std::env::args().nth(0).unwrap())
                {
                    native_dialog::MessageDialogBuilder::default()
                        .set_text(format!("Failed to enable auto backup: {}", err))
                        .alert()
                        .show();
                }
            }
            Err(err) => {
                native_dialog::MessageDialogBuilder::default()
                    .set_text(format!("Failed to enable auto backup: {}", err))
                    .alert()
                    .show()
                    .unwrap_or_default();
            }
        }
    });
    #[cfg(target_os = "linux")]
    enable_auto_backup_button.connect_clicked(|_| {
        native_dialog::MessageDialogBuilder::default()
            .set_text(t!("messages.enable-auto-backup-for-linux"))
            .alert()
            .show()
            .unwrap_or_default();
    });

    wrapper.append(&title(t!("pages.settings.basic")));
    wrapper.append(&b1);
    wrapper.append(&b2);
    wrapper.append(&b3);
    wrapper.append(&b4);
    wrapper.append(&enable_auto_backup_button);
    wrapper.append(
        &Label::builder()
            .halign(gtk4::Align::Start)
            .justify(gtk4::Justification::Left)
            .label(&t!("pages.settings.scan-root").to_string())
            .build(),
    );
    wrapper.append(&scan_root_input);

    wrapper.append(&add_scan_root_button);

    wrapper.append(&title(t!("pages.settings.advanced")));

    wrapper.append(&title(t!("pages.settings.experimental")));
    wrapper.append(&b6);

    save_button.connect_clicked(move |_| {
        let mut instance = Settings::instance();

        if let Ok(level) = compression_level_input.text().parse() {
            instance.compression_level = level;
        } else {
            DialogBuilder::message()
                .set_title(t!("message.failed-heck"))
                .set_text(t!(
                    "message.int-wanted",
                    entry = t!("pages.settings.compression-level")
                ))
                .alert();
        }
        if let Ok(duration) = daemon_backup_duration_input_box.text().parse() {
            instance.daemon_backup_duration = duration
        } else {
            DialogBuilder::message()
                .set_title(t!("message.failed-heck"))
                .set_text(t!(
                    "message.int-wanted",
                    entry = t!("pages.settings.daemon-backup-duration")
                ))
                .alert();
        }

        let password = pass_input_box.text();
        let password_cmd = pass_cmd_input_box.text();
        if !password.is_empty() && !password_cmd.is_empty() {
            DialogBuilder::message()
                .set_title(t!("message.failed-check"))
                .set_text(format!(
                    "{}: {}",
                    t!("messages.failed-check"),
                    t!(
                        "messages.cannot-enable-together",
                        a = t!("pages.settings.password"),
                        b = t!("pages.settings.password-command")
                    )
                ))
                .alert();
            return;
        }
        instance.password = if password.is_empty() {
            None
        } else {
            Some(password.to_string())
        };
        instance.password_cmd = if password_cmd.is_empty() {
            None
        } else {
            Some(password_cmd.to_string())
        };

        instance.scan_root = scan_root_input_buffer0
            .text(
                &scan_root_input_buffer0.start_iter(),
                &scan_root_input_buffer0.end_iter(),
                true,
            )
            .split("\n")
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>();
        instance.sync = sync_switch.state();
        instance.save();
    });

    wrapper.append(&save_button);
    wrapper
}
