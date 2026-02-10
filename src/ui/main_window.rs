use gtk4::{Application, ApplicationWindow, Notebook, prelude::*};

use crate::ui::pages::pages;
pub fn main_window(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("MineSave")
        .default_width(320)
        .default_height(200)
        .build();
    let mut notebook = Notebook::builder()
        .width_request(120)
        .tab_pos(gtk4::PositionType::Left)
        .build();

    pages(&mut notebook);
    window.set_child(Some(&notebook));
    window.present();
}
