use std::{borrow::Cow, path::PathBuf};

use gtk4::{
    Box, Button, Label, Switch, Text,
    prelude::{BoxExt, ButtonExt},
};

use crate::{settings::Settings, ui::pages::build_wrapper, utils::report_err};

pub fn settings() -> Box {
    let wrapper = build_wrapper();
    let save_button = Button::with_label(&t!("pages.settings.save").to_string());

    let (b1, sync_switch) = switch(t!("pages.settings.sync"), Settings::instance().sync);

    let add_scan_root_button = Button::with_label(&t!("pages.settings.add-scan-root").to_string());
    add_scan_root_button.connect_clicked(|_| {
        if let Ok(result) = native_dialog::FileDialogBuilder::default()
            .open_single_dir()
            .show()
            .inspect_err(report_err("Failed to open dialog"))
        {
            match result {
                Some(path) => {
                    Settings::instance().scan_root.push(path);
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
    wrapper.append(&add_scan_root_button);

    wrapper.append(&b1);

    save_button.connect_clicked(move |_| {
        let mut instance = Settings::instance();
        instance.sync = sync_switch.state();
        instance.save();
    });
    wrapper.append(&save_button);
    wrapper
}

fn text_input(label: Cow<str>, init_state: String) -> (Box, Text) {
    let b: Box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    b.append(&Label::builder().label(&label.to_string()).build());
    let text = Text::builder().text(init_state).build();
    b.append(&text);
    (b, text)
}

fn switch(label: Cow<str>, init_state: bool) -> (Box, Switch) {
    let b: Box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .build();
    b.append(&Label::builder().label(&label.to_string()).build());
    let switch = Switch::builder().state(init_state).build();
    b.append(&switch);
    (b, switch)
}
