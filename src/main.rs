extern crate wand;

use wand::cli_app::{build_cli_app, generator_sub_command};
use wand::generator::ProjectWand;

fn main() {
    env_logger::init();

    // cli app
    let mut app = build_cli_app();
    // add generator sub command
    app = app.subcommand(generator_sub_command());
    let matches = app.get_matches();

    match matches.subcommand() {
        ("generator", Some(_generator_command)) => {
            let mut wand_project = ProjectWand::new();
            wand_project.start();
        },
        _ => println!("{}", matches.usage()),
    }
}
