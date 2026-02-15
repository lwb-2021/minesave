use std::{cell::RefCell, sync::Arc};

use gtk4::{
    Box, Button, Image, Label, Window,
    glib::object::Cast,
    prelude::{BoxExt, ButtonExt, EditableExt, GtkWindowExt, WidgetExt},
};
use rustic_core::{SnapshotOptions, repofile::SnapshotFile};

use crate::{
    MINESAVE_DATA_HOME,
    backup::AppState,
    tasks::{self, TaskInfo},
    ui::{
        pages::build_wrapper,
        utils::{cardify, dialog_button_box, dialog_wrapper, title, with_label},
    },
};

pub fn saves() -> Box {
    let wrapper = build_wrapper();
    for (id0, save) in AppState::instance().saves.iter() {
        let save_card = Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();
        let save_card_left = Box::builder().margin_end(12).build();
        let save_card_right = Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        save_card_left.append(
            &Image::builder()
                .file(
                    MINESAVE_DATA_HOME
                        .join("resources")
                        .join(&id0)
                        .with_extension("png")
                        .to_string_lossy()
                        .to_string(),
                )
                .width_request(64)
                .height_request(64)
                .pixel_size(64)
                .build(),
        );

        let button_box = Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .halign(gtk4::Align::End)
            .spacing(8)
            .build();

        let backup_button = Button::with_label(&t!("pages.saves.backup").to_string());
        let recover_button = Button::with_label(&t!("pages.saves.recover").to_string());

        let for_id = id0.clone();

        backup_button.connect_clicked(move |_| {
            let cancelled = Arc::new(RefCell::new(true));

            let inner = dialog_wrapper();
            let (b1, label_input) = with_label::text_input(t!("pages.saves.label"), String::new());
            let (b2, description_input) =
                with_label::text_input(t!("pages.saves.description"), String::new());

            inner.append(&b1);
            inner.append(&b2);

            inner.append(&dialog_button_box(cancelled.clone()));

            let dialog = Window::builder()
                .title(t!("pages.saves.backup"))
                .child(&inner)
                .modal(true)
                .build();
            dialog.present();

            let for_id = for_id.clone();
            dialog.connect_close_request(move |_| {
                if cancelled.borrow().clone() {
                    return gtk4::glib::Propagation::Proceed;
                }

                tasks::spawn(
                    format!("{}: {}", t!("pages.saves.backup"), label_input.text()),
                    TaskInfo::Backup {
                        for_id: Some(for_id.clone()),
                        options: SnapshotOptions::default()
                            .label(label_input.text().to_string())
                            .description(description_input.text().to_string()),
                    },
                );
                gtk4::glib::Propagation::Proceed
            });
        });

        let id = id0.clone();

        recover_button.connect_clicked(move |_| {
            let inner = dialog_wrapper();
            inner.set_valign(gtk4::Align::Fill);

            let cancel_btn = Button::with_label(&t!("messages.cancel").to_string());

            cancel_btn.connect_clicked(|btn| {
                let window: Window = btn.root().unwrap().dynamic_cast().unwrap();
                window.close();
            });

            let save = &AppState::instance().saves[&id];

            for snapshot in save.list_backups().unwrap_or_default() {
                let snapshot_card = Box::builder().build();
                snapshot_card.append(&title(snapshot.label.clone()));
                let btn = Button::builder().child(&snapshot_card).build();

                let id = id.clone();

                let save_name = save.name.clone();

                btn.connect_clicked(move |_| {
                    tasks::spawn(
                        format!(
                            "{}: {}/{}",
                            t!("pages.saves.recover"),
                            save_name,
                            snapshot.label.clone()
                        ),
                        TaskInfo::Recover {
                            id: id.clone(),
                            snapshot: snapshot.clone(),
                        },
                    );
                });
                inner.append(&btn);
            }

            inner.append(&cancel_btn);

            let dialog = Window::builder()
                .title(t!("pages.saves.backup"))
                .child(&inner)
                .modal(true)
                .build();
            dialog.present();
        });

        button_box.append(&Label::builder().hexpand(true).build());
        button_box.append(&backup_button);
        button_box.append(&recover_button);
        save_card_right.append(&title(&save.name));
        save_card_right.append(&button_box);

        save_card.append(&save_card_left);
        save_card.append(&save_card_right);
        wrapper.append(&cardify(save_card));
    }

    wrapper
}

fn build_button_from_snapshot(_snapshot: &SnapshotFile) -> Button {
    let btn = Button::builder().build();
    btn
}
