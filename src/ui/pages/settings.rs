use gtk4::{
    Box, Button, Entry, Label, Switch, TextView,
    prelude::{BoxExt, ButtonExt, EditableExt, EntryExt, TextBufferExt, TextViewExt},
};
use native_dialog::DialogBuilder;
use std::{borrow::Cow, path::PathBuf};

use crate::{
    settings::Settings,
    ui::{pages::build_wrapper, utils::title},
    utils::report_err,
};

pub fn settings() -> Box {
    let wrapper = build_wrapper();
    let save_button = Button::builder()
        .halign(gtk4::Align::End)
        .label(&t!("pages.settings.save").to_string())
        .build();

    let (b1, pass_input_box) = text_input(
        t!("pages.settings.password"),
        Settings::instance().password.clone().unwrap_or_default(),
    );
    pass_input_box.set_visibility(false);
    let (b2, pass_cmd_input_box) = text_input(
        t!("pages.settings.password-command"),
        Settings::instance()
            .password_cmd
            .clone()
            .unwrap_or_default(),
    );

    let (b3, sync_switch) = switch(t!("pages.settings.sync"), Settings::instance().sync);

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

    wrapper.append(&title(t!("pages.settings.basic")));
    wrapper.append(&b1);
    wrapper.append(&b2);
    wrapper.append(
        &Label::builder()
            .halign(gtk4::Align::Start)
            .justify(gtk4::Justification::Left)
            .label(&t!("pages.settings.scan-root").to_string())
            .build(),
    );
    wrapper.append(&scan_root_input);

    wrapper.append(&add_scan_root_button);

    wrapper.append(&title(t!("pages.settings.experimental")));
    wrapper.append(&b3);

    save_button.connect_clicked(move |_| {
        let mut instance = Settings::instance();
        let password = pass_input_box.text();
        let password_cmd = pass_cmd_input_box.text();
        if !password.is_empty() && !password_cmd.is_empty() {
            DialogBuilder::message().set_title(format!(
                "{}: {}",
                t!("messages.failed-check"),
                t!(
                    "messages.cannot-enable-together",
                    a = t!("pages.settings.password"),
                    b = t!("pages.settings.password-command")
                )
            ));
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

fn text_input(label: Cow<str>, init_state: String) -> (Box, Entry) {
    let b: Box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    b.append(
        &Label::builder()
            .label(&label.to_string())
            .width_chars(16)
            .xalign(0.0)
            .build(),
    );
    let entry = Entry::builder().text(init_state).build();
    b.append(&entry);
    (b, entry)
}

fn switch(label: Cow<str>, init_state: bool) -> (Box, Switch) {
    let b: Box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    b.append(
        &Label::builder()
            .label(&label.to_string())
            .width_chars(16)
            .xalign(0.0)
            .build(),
    );
    let switch = Switch::builder().state(init_state).build();
    b.append(&switch);
    (b, switch)
}
