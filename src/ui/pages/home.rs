use gtk4::{
    Box, CssProvider, GridLayout, GridView, Label, Orientation, Widget,
    prelude::{BoxExt, GridExt, LayoutManagerExt, StyleContextExt, WidgetExt},
};

use crate::ui::{
    pages::build_wrapper,
    utils::{cardify, title},
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
        let b = Box::builder().orientation(Orientation::Vertical).build();
        b.append(&title("Work in progress"));
        b
    }));
    wrapper.append(&cards);
    wrapper
}
