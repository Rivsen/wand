use clap::{App, SubCommand};

pub fn build_cli_app() -> App<'static, 'static> {
    App::new("My Wand")
        .version("1.0.0")
        .about("A cli tool for developers")
        // .subcommand(generator_sub_command())
}

pub fn generator_sub_command() -> App<'static, 'static> {
    SubCommand::with_name("generator")
        .about("Project generator")
        .version("1.0.0")
}