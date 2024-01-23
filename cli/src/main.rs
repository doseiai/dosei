mod command;
mod config;

#[cfg(test)]
mod test;

use crate::config::Config;
use clap::{arg, Command};

fn cli() -> Command {
  Command::new("dctl")
    .about("A fictional versioning CLI")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(
      Command::new("login")
        .about("Clones repos")
        .arg(arg!(<REMOTE> "The remote to clone"))
        .arg_required_else_help(true),
    )
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  println!("{:?}", config);
  let matches = cli().get_matches();
  match matches.subcommand() {
    Some(("clone", sub_matches)) => {
      println!(
        "Cloning {}",
        sub_matches.get_one::<String>("REMOTE").expect("required")
      );
    }
    _ => unreachable!(),
  }
  Ok(())
}
