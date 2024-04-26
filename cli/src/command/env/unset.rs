use crate::config::Config;
use clap::{Arg, ArgMatches, Command};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};

pub fn command() -> Command {
  Command::new("unset")
    .about("Unset environment variables")
    .arg(
      Arg::new("name")
        .help("The env variable name")
        .index(1)
        .required(true),
    )
}

pub fn unset_env(arg_matches: &ArgMatches, _config: &'static Config) -> anyhow::Result<()> {
  let name = arg_matches.get_one::<String>("name").unwrap();

  let path = ".env";
  let file = File::open(path);
  let mut env_vars = BTreeMap::new();

  if let Ok(file) = file {
    let reader = BufReader::new(file);
    for line in reader.lines() {
      let line = line?;
      if let Some((key, value)) = line.split_once('=') {
        env_vars.insert(key.trim().to_string(), value.trim().to_string());
      }
    }
  }

  env_vars.remove(name);

  let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;

  for (key, value) in &env_vars {
    writeln!(file, "{}={}", key, value)?;
  }

  Ok(())
}
