extern crate clap;
extern crate tera;
extern crate wand;

use clap::{App};
use tera::{Tera, Context};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::{read_dir, File, DirEntry, create_dir_all};
use prettytable::{Table, Row, Cell};
use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Select, Input};
use std::path::PathBuf;
use wand::ProjectWand;

// const LOG_TARGET: &str = "wand";
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TemplateOption {
//     id: String,
//     name: String,
//     default: Option<String>,
//     required: bool,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Template {
//     id: String,
//     name: String,
//     options: Vec<TemplateOption>,
// }
//
// #[derive(Debug)]
// pub struct TemplateEntry {
//     template_id: String,
//     template: Template,
//     tera: Option<Tera>,
//     context: Option<Context>,
//     path: DirEntry,
//     target_path: Option<String>,
// }
//
// pub struct Project {
//     name: String,
//     template_id: Option<String>,
//     tera: Option<Tera>,
//     context: Option<Context>,
//     target_path: Option<String>,
// }
//
// impl Project {
//     pub fn new(name: String) -> Project {
//         Project {
//             name,
//             template_id: None,
//             tera: None,
//             context: None,
//             target_path: None,
//         }
//     }
//
//     pub fn get_mut_tera_and_context(&mut self) -> (&mut Tera, &mut Context) {
//         self.init_tera();
//         self.init_context();
//
//         return (self.tera.as_mut().unwrap(), self.context.as_mut().unwrap());
//     }
//
//     pub fn get_mut_context(&mut self) -> &mut Context {
//         self.init_context();
//         self.context.as_mut().unwrap()
//     }
//
//     pub fn context_insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
//         let context = self.get_mut_context();
//         context.insert(key, val);
//     }
// }
//
// impl TemplateEntry {
//     pub fn render(&mut self) {
//         let template_id = self.template_id.clone();
//         let mut target_path = match self.target_path.clone() {
//             None => panic!("Not set output directory"),
//             Some(t) => t,
//         };
//
//         let (tera, context) = self.get_mut_tera_and_context();
//
//         target_path = target_path.trim_end_matches("/").into();
//         target_path.push_str("/");
//         target_path.push_str(&template_id.clone());
//
//         match create_dir_all(target_path.clone()) {
//             Err(e) => panic!("Cannot create target path, {}", e),
//             _ => {},
//         }
//
//         for (template_name, template) in &tera.templates {
//
//             debug!(target: LOG_TARGET, "find a template: {:?}, {:?}", template_name, template);
//
//             let output_file = format!("{}/{}", &target_path, template.name.clone());
//             let output_file_path = PathBuf::from(output_file.clone());
//             let output_dir = output_file_path.parent().unwrap();
//
//             debug!(target: LOG_TARGET, "will put file at '{:?}'", output_file.clone());
//
//             if let Err(e) = create_dir_all(output_dir) {
//                 panic!("create directory failed: {:?}", e);
//             }
//
//             if let Err(e) = tera.render_to(template_name, context, File::create(output_file.clone()).unwrap()) {
//                 panic!("file '{:?}' render failed: {:?}", output_file.clone(), e);
//             }
//
//             info!(target: LOG_TARGET, "file '{:?}' rendered", output_file.clone());
//         }
//
//         println!("project '{:?}' generated at '{:?}' directory", template_id.clone(), target_path.clone());
//     }
//
//     pub fn init_tera(&mut self) -> Tera {
//         debug!(target: LOG_TARGET, "init tera template engine");
//
//         match Tera::new(&self.path.path().join("**/*").display().to_string()) {
//             Ok(t) => t,
//             Err(e) => {
//                 panic!("Parsing {:?} templates error(s): {}", self, e);
//             }
//         }
//     }
//
//     pub fn get_mut_tera(&mut self) -> &mut Tera {
//         self.init_tera();
//         self.tera.as_mut().unwrap()
//     }
//
//     pub fn get_tera(&mut self) -> &Tera {
//         self.init_tera();
//         self.tera.as_ref().unwrap()
//     }
// }
//
// #[derive(Debug)]
// pub struct TemplateEntryList {
//     keys: Vec<String>,
//     templates: HashMap<String, TemplateEntry>,
//     projects: HashMap<String, Project>,
//     internal_path: String,
//     external_paths: Vec<String>,
// }
//
// impl TemplateEntryList {
//     pub fn add_a_template_path(&mut self, path: String) {
//         let templates_dir = read_dir(path);
//
//         let templates_dir = match templates_dir {
//             Ok(t) => t,
//             Err(e) => panic!("Load templates error: {}", e),
//         };
//
//         for template_dir in templates_dir {
//             let template_dir = match template_dir {
//                 Ok(t) => t,
//                 Err(e) => panic!("Read files error: {}", e),
//             };
//
//             info!(target: LOG_TARGET, "load template: {:?}", template_dir.path());
//
//             if !template_dir.path().is_dir() {
//                 warn!(target: LOG_TARGET, "{:?} not a directory, continue", template_dir.path());
//                 continue;
//             }
//
//             let config_file_dir = template_dir.path().join("config.json");
//
//             debug!(target: LOG_TARGET, "read template config: {:?}", config_file_dir);
//
//             let config_file = File::open(config_file_dir);
//
//             if let Err(e) = config_file {
//                 panic!("Read files error: {}", e);
//             }
//
//             let template: Template = serde_json::from_reader(config_file.unwrap()).unwrap();
//
//             debug!(target: LOG_TARGET, "got a valid template: {:?}", template);
//
//             self.keys.push(template.id.clone());
//             self.templates.insert(template.id.clone(), TemplateEntry {
//                 template_id: template.id.clone(),
//                 template,
//                 path: template_dir,
//                 context: None,
//                 tera: None,
//                 target_path: Some("./output".into()),
//             });
//         }
//     }
//
//     pub fn start(&mut self) {
//         self.add_a_template_path(self.internal_path.clone());
//
//         for external_path in self.external_paths {
//             self.add_a_template_path(external_path.clone());
//         }
//
//         self.console_loop();
//     }
//
//     pub fn print_console_table(&self) {
//         let mut table = Table::new();
//         let templates = &self.templates;
//         table.add_row(row!["id", "Name", "Path"]);
//
//         for (_, template_entry) in templates.into_iter() {
//             table.add_row(Row::new(vec![
//                 Cell::new(&template_entry.template.id),
//                 Cell::new(&template_entry.template.name),
//                 Cell::new(&template_entry.path.path().display().to_string()),
//             ]));
//         }
//
//         table.printstd();
//     }
//
//     pub fn console_loop(&mut self) {
//         let theme = ColorfulTheme::default();
//
//         loop {
//             self.print_console_table();
//
//             let template_key = Select::with_theme(&theme)
//                 .with_prompt("Choose a template to start")
//                 .items(&self.keys)
//                 .default(0)
//                 .paged(true)
//                 .interact()
//                 .unwrap();
//
//             debug!(target: LOG_TARGET, "choose {:?}", self.keys.get(template_key));
//
//             if template_key == 0 {
//                 println!("Bye bye~");
//                 return;
//             }
//
//             let term = Term::buffered_stderr();
//             let template_entry_id = self.keys.get(template_key).unwrap().clone();
//             let template_entry = self.templates.get_mut(&template_entry_id).unwrap();
//
//             let project_name = Input::<String>::with_theme(&theme)
//                 .with_prompt("Type a name for new project")
//                 .interact_text_on(&term)
//                 .unwrap();
//
//             let mut project = Project::new(project_name);
//             project.template_id = Some(template_entry_id.clone());
//             project.tera = Some(template_entry.init_tera());
//             project.context = Some(Context::new());
//
//             println!("Now we will set some options before render template:");
//
//             for template_option in template_entry.template.options.clone().into_iter() {
//                 let default = match template_option.default {
//                     Some(value) => value,
//                     None => "".into(),
//                 };
//
//                 let value = Input::<String>::with_theme(&theme)
//                     .with_prompt(template_option.name.clone())
//                     .default(default.into())
//                     .interact_text_on(&term)
//                     .unwrap();
//
//                 template_entry.context_insert(template_option.id, &value);
//             }
//
//             template_entry.render();
//         }
//     }
// }

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

    // // template config
    // let mut template_list = TemplateEntryList {
    //     internal_path: "templates/".into(),
    //     external_paths: vec![],
    //     keys: vec!["Exit".into()],
    //     templates: HashMap::new(),
    //     projects: HashMap::new(),
    // };
    //
    // template_list.start();
}
