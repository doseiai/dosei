use crate::config::Config;
use crate::git::git_clone;
use clap::{Arg, ArgMatches, Command};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;
use std::{env, fs};
use tempfile::tempdir;

pub fn sub_command() -> Command {
  Command::new("new")
    .about("Create a new project from a template")
    .arg(
      Arg::new("template")
        .help("The Template name")
        .index(1)
        .value_parser(["fastapi"])
        .required(true),
    )
    .arg(
      Arg::new("destination")
        .help("The folder destination")
        .index(2),
    )
}

pub fn new(_: &'static Config, arg_matches: &ArgMatches) {
  let template = arg_matches.get_one::<String>("template").expect("required");
  let destination = arg_matches
    .get_one::<String>("destination")
    .unwrap_or(template);

  let selected_template = match template.to_lowercase().as_str() {
    "fastapi" => TemplateType::FastAPI.template(),
    _ => {
      eprintln!("Invalid template, Options: fastapi");
      return;
    }
  };
  println!("Creating a new project from the {} template", template);

  let temp_dir = tempdir().unwrap();
  let temp_path = temp_dir.path();
  let mut template_path = temp_path.to_path_buf();
  template_path.push(&selected_template.path);

  let _ = git_clone(
    "https://github.com/doseiai/examples",
    temp_path,
    Some("main"),
  );

  let mut target_path = env::current_dir().unwrap();
  target_path.push(destination);

  let start_copying = Instant::now();
  copy_dir_all(&template_path, &target_path).unwrap();
  let elapsed = start_copying.elapsed().as_secs_f64() * 1000.0;
  println!(
    "Copying {} completed: {:.2}ms",
    &selected_template.path, elapsed
  );

  Repository::init(&target_path).unwrap();
  println!(
    "Initialized empty Git repository in {}",
    &target_path.display()
  );

  println!();
  println!("That's it!");
  println!("Enter your project directory using cd {}", &destination);
  println!("Join the community at https://discord.gg/BP5aUkhcAh");
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

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
  if !dst.exists() {
    fs::create_dir_all(dst)?;
  }

  for entry in fs::read_dir(src)? {
    let entry = entry?;
    let ty = entry.file_type()?;
    let src_path = entry.path();
    let dst_path = dst.join(entry.file_name());

    if ty.is_dir() {
      copy_dir_all(&src_path, &dst_path)?;
    } else {
      fs::copy(&src_path, &dst_path)?;
    }
  }
  Ok(())
}
