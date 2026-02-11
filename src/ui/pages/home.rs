use gtk4::{
    Box, Button, GridLayout, Label, Orientation,
    prelude::{BoxExt, ButtonExt},
};
use native_dialog::DialogBuilder;
use rustic_core::SnapshotOptions;

use crate::{
    backup::AppState,
    ui::{
        pages::build_wrapper,
        utils::{cardify, title},
    },
};

pub fn home() -> Box {
    let wrapper: Box = build_wrapper();
    wrapper.append(&title(t!("pages.home.welcome")));
    let cards = Box::builder()
        .layout_manager(
            &GridLayout::builder()
                .column_spacing(8)
                .row_spacing(8)
                .build(),
        )
        .build();
    cards.append(&cardify({
        let backup_button = Button::with_label(&t!("pages.home.quick-backup").to_string());
        backup_button.connect_clicked(|_| {
            for save in AppState::instance().saves.values_mut() {
                if let Err(err) = save.run_backup(
                    SnapshotOptions::default().label(t!("pages.home.quick-backup").to_string()),
                ) {
                    debug!("backup_failed");
                    DialogBuilder::message()
                        .set_title(t!("messages.backup-failed"))
                        .set_text(format!("{}: {}", t!("messages.backup-failed"), err))
                        .alert();
                }
            }
            AppState::instance().save().unwrap_or_default();
        });

        let b = Box::builder()
            .width_request(320)
            .orientation(Orientation::Vertical)
            .build();
        b.append(&title(t!("pages.home.saves-summary")));
        b.append(&Label::builder().build());
        b.append(
            &Label::builder()
                .label(format!(
                    "{}: {}",
                    t!("pages.home.saves-count"),
                    AppState::instance().saves.len()
                ))
                .xalign(0.0)
                .build(),
        );
        b.append(&Label::builder().build());
        b.append(&backup_button);
        b
    }));
    wrapper.append(&cards);
    wrapper
}
