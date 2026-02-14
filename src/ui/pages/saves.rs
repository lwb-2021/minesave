use gtk4::{
    Box, Button, Image, Label,
    prelude::{BoxExt, ButtonExt},
};
use rustic_core::SnapshotOptions;

use crate::{
    MINESAVE_DATA_HOME,
    backup::AppState,
    ui::{
        pages::build_wrapper,
        utils::{cardify, title},
    },
};

pub fn saves() -> Box {
    let wrapper = build_wrapper();
    for (id, save) in AppState::instance().saves.iter().clone() {
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
                        .join(&id)
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

        // TODO
        let save_instance_for_backup = save.clone();
        backup_button.connect_clicked(|_| {});
        let save_instance_for_recover = save.clone();
        recover_button.connect_clicked(|_| {});

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
