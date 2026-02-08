#[macro_use]
extern crate rust_i18n;
#[macro_use]
extern crate log;

use env_logger::Env;

mod backup;
mod settings;
mod ui;
mod utils;

i18n!();

fn main() {
    env_logger::init_from_env(Env::default());
    rust_i18n::set_locale(&sys_locale::get_locale().unwrap_or_else(|| String::from("en-US")));
    ui::run_app();
}
