use std::{cell::RefCell, sync::Arc};

use gtk4::{
    Box, Button, CssProvider, Label, Window,
    glib::object::Cast,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, StyleContextExt, WidgetExt},
};

#[inline]
pub fn title<S: std::fmt::Display>(content: S) -> Label {
    Label::builder()
        .use_markup(true)
        .label(format!("<span size='x-large'>{}</span>", content))
        .justify(gtk4::Justification::Left)
        .halign(gtk4::Align::Start)
        .build()
}
pub fn cardify(target: Box) -> Box {
    let css_provider = CssProvider::builder().build();
    css_provider.load_from_data(
        ".card { border-radius: 8px; padding: 12px; background-color: rgba(127, 0, 255, 0.08); }",
    );
    target.add_css_class("card");
    target
        .style_context()
        .add_provider(&css_provider, 2147483646);
    target
}

#[inline]
pub fn dialog_wrapper() -> Box {
    let b = Box::builder()
        .spacing(12)
        .orientation(gtk4::Orientation::Vertical)
        .build();
    b.add_css_class("wrapper");
    let css_provider = CssProvider::builder().build();
    css_provider.load_from_data(".wrapper { padding: 24px; }");
    b.style_context().add_provider(&css_provider, 0);
    b
}

pub fn dialog_button_box(cancelled: Arc<RefCell<bool>>) -> Box {
    let wrapper = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .build();
    let ok = Button::with_label(&t!("messages.ok").to_string());
    let cancel = Button::with_label(&t!("messages.cancel").to_string());
    let cancelled1 = cancelled.clone();
    ok.connect_clicked(move |btn| {
        *cancelled1.borrow_mut() = false;
        let w: Window = btn.root().unwrap().dynamic_cast().unwrap();
        w.close();
    });
    let cancelled2 = cancelled.clone();
    cancel.connect_clicked(move |btn| {
        *cancelled2.borrow_mut() = false;
        let w: Window = btn.root().unwrap().dynamic_cast().unwrap();
        w.close();
    });
    wrapper.append(&ok);
    wrapper.append(&cancel);
    wrapper
}

pub mod with_label {
    use std::borrow::Cow;

    use gtk4::{Box, Entry, Label, Switch, prelude::BoxExt};

    pub fn text_input(label: Cow<str>, init_state: String) -> (Box, Entry) {
        let b: Box = Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();
        b.append(&build_label(label));
        let entry = Entry::builder().text(init_state).width_chars(48).build();
        b.append(&entry);
        (b, entry)
    }

    pub fn switch(label: Cow<str>, init_state: bool) -> (Box, Switch) {
        let b: Box = Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();
        b.append(&build_label(label));
        let switch = Switch::builder().state(init_state).build();
        b.append(&switch);
        (b, switch)
    }

    fn build_label(label: Cow<str>) -> Label {
        Label::builder()
            .label(&label.to_string())
            .width_chars(16)
            .xalign(0.0)
            .build()
    }
}
