use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use git2::Repository;
use std::path::Path;
use std::time::Instant;
use std::{env, fs};
use tempfile::tempdir;

pub fn command() -> Command {
  Command::new("new")
    .about("Create a new service from a template")
    .arg(
      Arg::new("template")
        .help("The Template name")
        .index(1)
        .value_parser(["express"])
        .required(true),
    )
    .arg(
      Arg::new("destination")
        .help("The folder destination")
        .index(2),
    )
}

pub fn new(arg_matches: &ArgMatches) -> anyhow::Result<()> {
  let template = arg_matches.get_one::<String>("template").unwrap();
  let destination = arg_matches
    .get_one::<String>("destination")
    .unwrap_or(template);
  println!("Creating a new service from the {} template", template);

  let temp_dir = tempdir().context("Failed to create temporary directory.")?;
  let temp_path = temp_dir.path();
  let template_path = temp_path.to_path_buf().join(template);

  let _ = dosei_util::git::clone(
    "https://github.com/doseiai/templates",
    temp_path,
    Some("main"),
  );

  let target_path = env::current_dir()?.join(destination);

  let start_copying = Instant::now();
  copy_dir_all(&template_path, &target_path).unwrap();
  let elapsed = start_copying.elapsed().as_secs_f64() * 1000.0;
  println!("Copying {} completed: {:.2}ms", template, elapsed);

  Repository::init(&target_path).unwrap();
  println!(
    "Initialized empty Git repository in {}",
    &target_path.display()
  );

  println!();
  println!("That's it!");
  println!("Enter your project directory using: cd {}", &destination);
  println!("Join the community at https://discord.gg/BP5aUkhcAh");
  Ok(())
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
