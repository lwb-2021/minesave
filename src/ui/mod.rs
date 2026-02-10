use crate::utils::report_err;
use gtk4::{
    Application,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    glib::ExitCode,
};

mod main_window;
mod pages;
mod utils;

const APP_ID: &str = "io.github.lwb-2021.MineSave";
pub fn run_app() -> ExitCode {
    if let Err(_) = gtk4::init().inspect_err(report_err("Failed to init UI")) {
        return ExitCode::FAILURE;
    }

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(main_window::main_window);
    app.run()
}
