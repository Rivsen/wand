use std::collections::HashMap;
use tera::{Tera, Context};
use std::fs::{read_dir, File};

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

pub struct TemplateConfig {
    id: String,
    name: String,
    options: Vec<TemplateEntryOption>,
}

pub struct TemplateEntryOption {
    id: String,
    name: String,
    default: Option<String>,
    required: bool,
}

pub struct ProjectWand {
    template_list: TemplateEntryList,
    projects: Vec<Project>,
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

impl ProjectWand {
    pub fn new() -> ProjectWand {
        ProjectWand {
            internal_path: "templates/".into(),
            external_paths: vec![],
            template_list: TemplateEntryList {
                keys: vec![],
                templates: HashMap::new(),
            },
            projects: vec![],
        }
    }

    pub fn init(&mut self) {
        self.template_list.add_template_path(self.internal_path.clone());

        for external_path in self.external_paths {
            self.template_list.add_template_path(external_path.clone());
        }
    }

    pub fn console_loop() {
    }

    pub fn start(&mut self) {
        self.init();
        self.console_loop();
    }
}

impl TemplateEntryList {
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
                panic!("Read files error: {}", e);
            }

            let template: TemplateConfig = serde_json::from_reader(config_file.unwrap()).unwrap();

            debug!(target: LOG_TARGET, "got a valid template: {:?}", template);

            self.keys.push(template.id.clone());
            self.templates.insert(template.id.clone(), TemplateEntry {
                id: template.id.clone(),
                name: template.name.clone(),
                config: template,
                path: template_dir.into(),
            });
        }
    }
}