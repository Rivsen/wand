#[macro_use]
extern crate prettytable;
extern crate clap;
extern crate tera;

use clap::{App};
use tera::{Tera, Context};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::{read_dir, File, DirEntry};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use prettytable::{Table, Row, Cell};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateOption {
    id: String,
    name: String,
    default: Option<String>,
    required: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Template {
    id: String,
    name: String,
    options: Vec<TemplateOption>,
}

#[derive(Debug)]
pub struct TemplateEntry {
    template: Template,
    tera: Option<Tera>,
    context: Option<Context>,
    path: DirEntry,
}

fn build_cli_app() -> App<'static, 'static> {
    App::new("My Wand")
}

fn ask(question: String) -> Option<String> {
    let mut rl = Editor::<()>::new();

    if rl.load_history("history.txt").is_err() {
        // println!("No previous history.");
    }

    println!("{}", question);
    let readline = rl.readline(">> ");

    let answer = match readline {
        Ok(line) => {
            rl.add_history_entry(line.as_str());
            Some(line)
        },
        Err(ReadlineError::Interrupted) => {
            println!("CTRL-C");
            None
        },
        Err(ReadlineError::Eof) => {
            println!("CTRL-D");
            None
        },
        Err(err) => {
            println!("Error: {:?}", err);
            None
        }
    };

    rl.save_history("history.txt").unwrap();

    answer
}

fn build_template(template_entry: &mut TemplateEntry) {
    // template render
    let mut context = Context::new();
    println!("Now we will set some options before render template");

    for template_option in template_entry.template.options.clone().into_iter() {
        let value = match ask(template_option.name.clone()) {
            Some(value) => {
                println!("value is '{}'", value);
                value
            },
            None => {
                println!("no value set, using default: '{:?}'", template_option.default);

                match template_option.default {
                    Some(value) => value,
                    None => "".into(),
                }
            },
        };

        context.insert(template_option.id, &value);
    }

    let tera = match Tera::new(&template_entry.path.path().join("**/*").display().to_string()) {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };

    template_entry.tera = Some(tera);
    template_entry.context = Some(context);

    println!("{:?}", template_entry);
}

fn generate_loop(templates: &mut HashMap<String, TemplateEntry>) {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut table = Table::new();
    table.add_row(row!["id", "Name", "Path"]);

    for (_, template_entry) in templates.into_iter() {
        table.add_row(Row::new(vec![
            Cell::new(&template_entry.template.id),
            Cell::new(&template_entry.template.name),
            Cell::new(&template_entry.path.path().display().to_string()),
        ]));
    }

    loop {
        table.printstd();

        match ask("Choose a template".into()) {
            Some(template_key) => {
                println!("choose '{}'", template_key);

                let template_entry = templates.get_mut(&template_key).unwrap();
                build_template(template_entry);

            },
            None => {
                println!("nothing choose, exit");
                break;
            }
        }
    }
}

fn main() {
    // cli app
    let app = build_cli_app();
    app.get_matches();

    // template config
    let mut templates: HashMap<String, TemplateEntry> = HashMap::new();
    let base_path = "templates/";
    let templates_dir = read_dir(base_path);

    let templates_dir = match templates_dir {
        Ok(t) => t,
        Err(e) => panic!("Load templates error: {}", e),
    };

    for template_dir in templates_dir {
        let template_dir = match template_dir {
            Ok(t) => t,
            Err(e) => panic!("Read files error: {}", e),
        };

        println!("template: {:?}", template_dir.path());

        if !template_dir.path().is_dir() {
            println!("{:?} not a directory, continue", template_dir.path());

            continue;
        }

        let config_file_dir = template_dir.path().join("config.json");
        println!("{:?}", config_file_dir);
        let config_file = File::open(config_file_dir);

        if let Err(e) = config_file {
            panic!("Read files error: {}", e);
        }

        let template: Template = serde_json::from_reader(config_file.unwrap()).unwrap();
        println!("{:?}", template);

        templates.insert(template.id.clone(), TemplateEntry {
            template,
            path: template_dir,
            context: None,
            tera: None,
        });
    }

    println!("{:?}", templates);
    generate_loop(&mut templates);

    // template engine
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            panic!("Parsing error(s): {}", e);
        }
    };

    // template render
    let mut context = Context::new();
    context.insert("server_workers", &16);

    let test_env = tera.render("actix-web/.env.example", &context);
    println!("{:?}", test_env);
}
