use crate::config::Config;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn new(config: &'static Config, arg_matches: &ArgMatches) {
  let template = arg_matches.get_one::<String>("template").expect("required");
  let name = arg_matches.get_one::<String>("name").expect("required");
  println!("{} {}", template, name);

  let selected_template = match template.to_lowercase().as_str() {
    "fastapi" => TemplateType::FastAPI.template(),
    _ => panic!("Invalid template"),
  };
  let mut body = json!({"name": name});
  merge(&mut body, serde_json::to_value(selected_template).unwrap());

  match config
    .cluster_api_client()
    .expect("Client connection failed")
    .post(format!("{}/projects/clone", config.api_base_url))
    .bearer_auth(config.bearer_token())
    .json(&body)
    .send()
  {
    Ok(response) => {
      if response.status().is_success() {
        println!("Project created successfully.");
      } else {
        eprintln!("Failed to create project: {:?}", response.text().unwrap());
      }
    }
    Err(e) => eprintln!("Failed to send request: {:?}", e),
  }
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
        source_full_name: "doseiai/dosei".to_string(),
        path: "examples/fastapi".to_string(),
        branch: "main".to_string(),
      },
    }
  }
}

fn merge(a: &mut Value, b: Value) {
  match (a, b) {
    (a @ &mut Value::Object(_), Value::Object(b)) => {
      let a = a.as_object_mut().unwrap();
      for (k, v) in b {
        merge(a.entry(k).or_insert(Value::Null), v);
      }
    }
    (a, b) => *a = b,
  }
}
