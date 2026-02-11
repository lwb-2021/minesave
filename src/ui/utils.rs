use gtk4::{
    Box, CssProvider, Label,
    prelude::{StyleContextExt, WidgetExt},
};

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
