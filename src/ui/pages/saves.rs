use gtk4::{Box, Image, prelude::BoxExt};

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
    for (id, save) in AppState::instance().saves.iter() {
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
        save_card_right.append(&title(&save.name));
        save_card.append(&save_card_left);
        save_card.append(&save_card_right);
        wrapper.append(&cardify(save_card));
    }
    wrapper
}
