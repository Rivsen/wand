
use std::collections::HashMap;
use tera::{Tera, Context};
use std::fs::{read_dir, File, create_dir_all};
use dialoguer::theme::ColorfulTheme;
use prettytable::{Table, Row, Cell};
use dialoguer::{Select, Input};
use console::Term;
use log::{info, warn, debug};
use std::ops::Add;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::borrow::{Borrow};

pub const LOG_TARGET: &str = "wand";

pub struct TemplateEntryList {
    keys: Vec<String>,
    templates: HashMap<String, TemplateEntry>,
}

pub struct TemplateEntry {
    id: String,
    name: String,
    path: String,
    config: TemplateConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateConfig {
    id: String,
    name: String,
    options: Vec<TemplateEntryOption>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateEntryOption {
    id: String,
    name: String,
    default: Option<String>,
    required: bool,
}

pub struct ProjectWand {
    template_list: TemplateEntryList,
    projects: HashMap<String, Project>,
    internal_path: String,
    external_paths: Vec<String>,
}

pub struct Project {
    name: String,
    template_id: Option<String>,
    tera: Option<Tera>,
    context: Option<Context>,
    output_path: Option<String>,
}

impl Project {
    fn new(name: String) -> Project {
        Project {
            name,
            template_id: None,
            tera: None,
            context: None,
            output_path: Some("./output".to_string()),
        }
    }

    pub fn init_tera(&mut self, path: String) {
        let path = path.trim_end_matches("/").to_string().add("**/*");

        self.tera = Some(match Tera::new(&path) {
            Ok(t) => t,
            Err(e) => panic!("Parsing {:?} templates error(s): {}", path, e),
        });
    }

    pub fn init_context(&mut self) {
        self.context = Some(Context::new());
    }

    pub fn get_mut_context(&mut self) -> &mut Context {
        self.init_context();
        self.context.as_mut().unwrap()
    }

    pub fn get_mut_tera_and_context(&mut self) -> (&mut Tera, &mut Context) {
        return (self.tera.as_mut().unwrap(), self.context.as_mut().unwrap());
    }

    pub fn context_insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        let context = self.get_mut_context();
        context.insert(key, val);
    }

    pub fn render(&mut self) {
        let template_id = self.template_id.clone().unwrap();
        let project_name = self.name.clone();
        let mut output_path = match self.output_path.clone() {
            None => panic!("Not set output directory"),
            Some(t) => t,
        };

        let (tera, context) = self.get_mut_tera_and_context();

        output_path = output_path.trim_end_matches("/").into();
        output_path.push_str("/");
        output_path.push_str(&project_name.clone());

        match create_dir_all(output_path.clone()) {
            Err(e) => panic!("Cannot create target path, {}", e),
            _ => {},
        }

        for (template_name, template) in &tera.templates {

            debug!(target: LOG_TARGET, "find a template: {:?}, {:?}", template_name, template);

            let output_file = format!("{}/{}", &output_path, template_name.clone());
            let output_file_path = PathBuf::from(output_file.clone());
            let output_dir = output_file_path.parent().unwrap();

            debug!(target: LOG_TARGET, "will put file at '{:?}'", output_file.clone());

            if let Err(e) = create_dir_all(output_dir) {
                panic!("create directory failed: {:?}", e);
            }

            if let Err(e) = tera.render_to(template_name, context, File::create(output_file.clone()).unwrap()) {
                panic!("file '{:?}' render failed: {:?}", output_file.clone(), e);
            }

            info!(target: LOG_TARGET, "file '{:?}' rendered", output_file.clone());
        }

        println!("project '{:?}' generated at '{:?}' directory", template_id.clone(), output_path.clone());
    }
}

impl ProjectWand {
    pub fn new() -> ProjectWand {
        ProjectWand {
            internal_path: "templates/".into(),
            external_paths: vec![],
            template_list: TemplateEntryList::new(),
            projects: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.template_list = TemplateEntryList::new();
        self.template_list.add_template_path(self.internal_path.clone());

        for external_path in self.external_paths.iter() {
            self.template_list.add_template_path(external_path.clone());
        }
    }

    pub fn print_console_templates_table(&self) {
        let mut table = Table::new();
        table.add_row(row!["id", "Name", "Path"]);

        for (_, template_entry) in self.template_list.templates.borrow().into_iter() {
            table.add_row(Row::new(vec![
                Cell::new(&template_entry.id),
                Cell::new(&template_entry.name),
                Cell::new(&template_entry.path),
            ]));
        }

        table.printstd();
    }

    pub fn print_console_projects_table(&self) {
        let mut table = Table::new();
        table.add_row(row!["Name", "Template", "Output", "Context"]);

        for (project_name, project) in self.projects.borrow().into_iter() {
            table.add_row(Row::new(vec![
                Cell::new(&project_name),
                Cell::new(&project.template_id.clone().unwrap_or("Not Set".into())),
                Cell::new(&project.output_path.clone().unwrap_or("Not Set".into())),
                Cell::new(&format!("{:?}", &project.context)),
            ]));
        }

        table.printstd();
    }

    pub fn console_loop(&mut self) {
        let theme = ColorfulTheme::default();

        loop {
            println!("\nConfigured Projects");
            self.print_console_projects_table();
            println!("\nLoaded templates");
            self.print_console_templates_table();
            let term = Term::buffered_stderr();

            let project_name = Input::<String>::with_theme(&theme)
                .with_prompt("Type a name for new project")
                .interact_text_on(&term)
                .unwrap();

            let template_index = Select::with_theme(&theme)
                .with_prompt("Choose a template to start")
                .items(&self.template_list.keys)
                .default(0)
                .paged(true)
                .interact_on(&term)
                .unwrap();

            debug!(target: LOG_TARGET, "choose {:?}", self.template_list.keys.get(template_index));

            if template_index == 0 {
                println!("Bye bye~");
                return;
            }

            let template_entry_id = self.template_list.keys.get(template_index).unwrap().clone();
            let template_entry = self.template_list.templates.get_mut(&template_entry_id).unwrap();

            let mut project = Project::new(project_name.clone());
            project.template_id = Some(template_entry_id.clone());
            project.init_tera(template_entry.path.clone());
            project.init_context();

            println!("Now we will set some options before render template:");

            for template_option in template_entry.config.options.clone().into_iter() {
                let default = match template_option.default {
                    Some(value) => value,
                    None => "".into(),
                };

                let value = Input::<String>::with_theme(&theme)
                    .with_prompt(template_option.name.clone())
                    .default(default.into())
                    .interact_text_on(&term)
                    .unwrap();

                project.context_insert(template_option.id, &value);
            }

            project.render();

            self.projects.insert(project_name.clone(), project);
        }
    }

    pub fn start(&mut self) {
        self.init();
        self.console_loop();
    }
}

impl TemplateEntryList {
    pub fn new() -> TemplateEntryList {
        TemplateEntryList {
            keys: vec!["Exit".into()],
            templates: HashMap::new(),
        }
    }

    pub fn add_template_path(&mut self, path: String) {
        let templates_dir = read_dir(path);

        let templates_dir = match templates_dir {
            Ok(t) => t,
            Err(e) => panic!("Load templates error: {}", e),
        };

        for template_dir in templates_dir {
            let template_dir = match template_dir {
                Ok(t) => t,
                Err(e) => panic!("Read files error: {}", e),
            };

            info!(target: LOG_TARGET, "load template: {:?}", template_dir.path());

            if !template_dir.path().is_dir() {
                warn!(target: LOG_TARGET, "{:?} not a directory, continue", template_dir.path());
                continue;
            }

            let config_file_dir = template_dir.path().join("config.json");

            debug!(target: LOG_TARGET, "read template config: {:?}", config_file_dir);

            let config_file = File::open(config_file_dir);

            if let Err(e) = config_file {
                panic!("Read config file error: {}", e);
            }

            let config: TemplateConfig = serde_json::from_reader(config_file.unwrap()).unwrap();

            debug!(target: LOG_TARGET, "got a valid template: {:?}", config);

            if self.templates.contains_key(&config.id.clone()) {
                println!("template '{}' loaded, skip", config.id.clone());
                continue;
            }

            self.keys.push(config.id.clone());
            self.templates.insert(config.id.clone(), TemplateEntry {
                id: config.id.clone(),
                name: config.name.clone(),
                config,
                path: template_dir.path().display().to_string(),
            });
        }
    }
}