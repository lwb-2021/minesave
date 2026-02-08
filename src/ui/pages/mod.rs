use gtk4::{
    CssProvider, Image, Notebook,
    prelude::{StyleContextExt, WidgetExt},
};

mod home;
mod settings;

pub fn pages(notebook: &mut Notebook) {
    notebook.append_page(
        &home::home(),
        Some(&Image::from_icon_name("go-home-symbolic")),
    );
    notebook.append_page(
        &settings::settings(),
        Some(&Image::from_icon_name("settings")),
    );
}

pub fn build_wrapper() -> gtk4::Box {
    let b = gtk4::Box::builder()
        .spacing(12)
        .width_request(400)
        .height_request(300)
        .orientation(gtk4::Orientation::Vertical)
        .build();
    b.add_css_class("wrapper");
    let css_provider = CssProvider::builder().build();
    css_provider.load_from_data(".wrapper { padding: 24px; }");
    b.style_context().add_provider(&css_provider, 0);
    b
}
