extern crate clap;
extern crate tera;
extern crate wand;

use clap::{App};
use wand::ProjectWand;

fn build_cli_app() -> App<'static, 'static> {
    App::new("My Wand")
}

fn main() {
    env_logger::init();

    // cli app
    let app = build_cli_app();
    app.get_matches();
    let mut wand_project = ProjectWand::new();

    wand_project.start();
}
