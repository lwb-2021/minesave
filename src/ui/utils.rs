use gtk4::{
    Box, CssProvider, Label, Orientation, Widget,
    prelude::{BoxExt, StyleContextExt, WidgetExt},
};

pub fn title<S: std::fmt::Display>(content: S) -> Label {
    Label::builder()
        .use_markup(true)
        .label(format!("<span size='x-large'>{}</span>", content))
        .build()
}
pub fn cardify(target: Box) -> Box {
    let css_provider = CssProvider::builder().build();
    css_provider.load_from_data(".card { border-radius: 8px; padding: 8px; }");
    target.add_css_class("card");
    target.style_context().add_provider(&css_provider, 0);
    target
}
