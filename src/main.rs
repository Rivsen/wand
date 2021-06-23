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
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Select, Input};
use std::borrow::Borrow;

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

#[derive(Debug)]
pub struct TemplateEntryList {
    keys: Vec<String>,
    templates: HashMap<String, TemplateEntry>,
}

impl TemplateEntryList {
    pub fn print_console_table(&self) {
        let mut table = Table::new();
        let templates = &self.templates;
        table.add_row(row!["id", "Name", "Path"]);

        for (_, template_entry) in templates.into_iter() {
            table.add_row(Row::new(vec![
                Cell::new(&template_entry.template.id),
                Cell::new(&template_entry.template.name),
                Cell::new(&template_entry.path.path().display().to_string()),
            ]));
        }

        table.printstd();
    }
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

fn print_templates_table(templates: &mut HashMap<String, TemplateEntry>) {
    let mut table = Table::new();
    table.add_row(row!["id", "Name", "Path"]);

    for (_, template_entry) in templates.into_iter() {
        table.add_row(Row::new(vec![
            Cell::new(&template_entry.template.id),
            Cell::new(&template_entry.template.name),
            Cell::new(&template_entry.path.path().display().to_string()),
        ]));
    }

    table.printstd();
}

fn console_loop(template_list: &mut TemplateEntryList) {
    let theme = ColorfulTheme::default();

    loop {
        template_list.print_console_table();

        let template_key = Select::with_theme(&theme)
            .with_prompt("Choose a template to start")
            .items(&template_list.keys)
            .default(0)
            .paged(true)
            .interact()
            .unwrap();

        println!("choose {:?}", template_list.keys.get(template_key));

        let template_entry_id = template_list.keys.get(template_key).unwrap().clone();
        let mut template_entry = template_list.templates.get_mut(&template_entry_id).unwrap();
        let mut context = Context::new();
        let term = Term::buffered_stderr();

        println!("Now we will set some options before render template");

        for template_option in template_entry.template.options.clone().into_iter() {
            let default = match template_option.default {
                Some(value) => value,
                None => "".into(),
            };

            let value = Input::with_theme(&theme)
                .with_prompt(template_option.name.clone())
                .default(&default)
                .interact_text_on(&term)
                .unwrap();

            context.insert(template_option.id, value);
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
}

fn main() {
    // cli app
    let app = build_cli_app();
    app.get_matches();

    // template config
    let mut templates: HashMap<String, TemplateEntry> = HashMap::new();
    let mut template_list = TemplateEntryList {
        keys: vec![],
        templates: HashMap::new(),
    };

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

        template_list.keys.push(template.id.clone());
        template_list.templates.insert(template.id.clone(), TemplateEntry {
            template,
            path: template_dir,
            context: None,
            tera: None,
        });

    }

    println!("{:?}", template_list);
    console_loop(&mut template_list);

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
