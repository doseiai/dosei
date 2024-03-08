use crate::config::Config;
use clap::{Arg, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn sub_command() -> Command {
  Command::new("create")
    .about("Create a new project from a template")
    .arg(
      Arg::new("template")
        .help("The Template name")
        .index(1)
        .required(true),
    )
}

pub fn create(config: &'static Config, arg_matches: &ArgMatches) {
  let template = arg_matches.get_one::<String>("template").expect("required");
  println!("{}", template);

  let selected_template = match template.to_lowercase().as_str() {
    "fastapi" => TemplateType::FastAPI.template(),
    _ => panic!("Invalid template"),
  };
  println!("{:?}", selected_template);
}

#[derive(Debug, Serialize, Deserialize)]
struct Template {
  source_full_name: String,
  path: String,
  branch: String,
}

enum TemplateType {
  FastAPI,
}

impl TemplateType {
  fn template(&self) -> Template {
    match self {
      TemplateType::FastAPI => Template {
        source_full_name: "doseiai/examples".to_string(),
        path: "examples/fastapi".to_string(),
        branch: "main".to_string(),
      },
    }
  }
}
