extern crate clap;
extern crate tera;

use clap::{App};
use tera::{Tera, Context};

fn build_cli_app() -> App<'static, 'static> {
    App::new("My Wand")
}

fn main() {
    let app = build_cli_app();
    app.get_matches();

    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };

    let mut context = Context::new();
    context.insert("server_workers", &16);

    let test_env = tera.render("actix-web/.env.example", &context);
    println!("{:?}", test_env);
}
