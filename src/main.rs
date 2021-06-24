#[macro_use]
extern crate prettytable;
extern crate clap;
extern crate tera;

use clap::{App};
use tera::{Tera, Context};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::{read_dir, File, DirEntry, create_dir_all};
use prettytable::{Table, Row, Cell};
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Select, Input};
use std::borrow::BorrowMut;
use std::ops::Index;
use std::path::PathBuf;

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
    template_id: String,
    template: Template,
    tera: Option<Tera>,
    context: Option<Context>,
    path: DirEntry,
    target_path: Option<String>,
}

impl TemplateEntry {
    pub fn render(&mut self) {
        let template_id = self.template_id.clone();
        let target_path = match self.target_path.clone() {
            None => panic!("Not set output directory"),
            Some(t) => t,
        };

        let (tera, context) = self.get_mut_tera_and_context();

        match create_dir_all(target_path.clone()) {
            Err(e) => panic!("Cannot create target path, {}", e),
            _ => {},
        }

        for (template_name, template) in &tera.templates {
            //println!("{:?}, {:?}", template_name, template);
            let output_file = format!("{}/{}/{}", &target_path, &template_id, template.path.as_ref().unwrap());
            let output_file_path = PathBuf::from(output_file);
            let output_dir = output_file_path.parent().unwrap();

            println!("{:?}", output_dir);

            create_dir_all(output_dir);
        }

        // let env_target = tera.render(".env.example", context);
        //
        // println!("{:?}", env_target);
        //
        // env_target.ok()
    }

    pub fn init_tera(&mut self) {
        if let Some(_) = self.tera {
            return;
        }

        let tera = match Tera::new(&self.path.path().join("**/*").display().to_string()) {
            Ok(t) => t,
            Err(e) => {
                panic!("Parsing {:?} templates error(s): {}", self, e);
            }
        };

        self.tera = Some(tera);
    }

    pub fn init_context(&mut self) {
        if let Some(_) = self.context {
            return;
        }

        self.context = Some(Context::new());
    }

    pub fn get_mut_tera(&mut self) -> &mut Tera {
        self.init_tera();
        self.tera.as_mut().unwrap()
    }

    pub fn get_tera(&mut self) -> &Tera {
        self.init_tera();
        self.tera.as_ref().unwrap()
    }

    pub fn get_mut_context(&mut self) -> &mut Context {
        self.init_context();
        self.context.as_mut().unwrap()
    }

    pub fn get_mut_tera_and_context(&mut self) -> (&mut Tera, &mut Context) {
        self.init_tera();
        self.init_context();

        return (self.tera.as_mut().unwrap(), self.context.as_mut().unwrap());
    }

    pub fn context_insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        let mut context = self.get_mut_context();
        context.insert(key, val);
    }
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

fn console_loop(template_list: &mut TemplateEntryList) {
    let theme = ColorfulTheme::default();

    loop {
        template_list.print_console_table();

        let mut template_key = Select::with_theme(&theme)
            .with_prompt("Choose a template to start")
            .items(&template_list.keys)
            .default(0)
            .paged(true)
            .interact()
            .unwrap();

        println!("choose {:?}", template_list.keys.get(template_key));

        if template_key == 0 {
            println!("Byebye~");

            return;
        }

        template_key -= 1;

        let template_entry_id = template_list.keys.get(template_key).unwrap().clone();
        let mut template_entry = template_list.templates.get_mut(&template_entry_id).unwrap();
        let term = Term::buffered_stderr();

        println!("Now we will set some options before render template");

        for template_option in template_entry.template.options.clone().into_iter() {
            let default = match template_option.default {
                Some(value) => value,
                None => "".into(),
            };

            let value = Input::<String>::with_theme(&theme)
                .with_prompt(template_option.name.clone())
                .default(default.into())
                .interact_text_on(&term)
                .unwrap();

            template_entry.context_insert(template_option.id, &value);
        }

        template_entry.render();

        println!("{:?}", &template_entry);
    }
}

fn main() {
    // cli app
    let app = build_cli_app();
    app.get_matches();

    // template config
    let mut templates: HashMap<String, TemplateEntry> = HashMap::new();
    let mut template_list = TemplateEntryList {
        keys: vec!["Exit".into()],
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
            template_id: template.id.clone(),
            template,
            path: template_dir,
            context: None,
            tera: None,
            target_path: Some("./output".into()),
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
