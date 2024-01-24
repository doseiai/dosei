mod command;
mod config;

#[cfg(test)]
mod test;

use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::session::session;
use crate::config::{Config, VERSION};
use clap::Command;

fn cli() -> Command {
  Command::new("dctl")
    .version(VERSION)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(Command::new("login").about("Log in to a cluster"))
    .subcommand(Command::new("logout").about("Log out from a cluster"))
    .subcommand(Command::new("session").about("Print active cluster session"))
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  let matches = cli().get_matches();
  match matches.subcommand() {
    Some(("login", _)) => login(config),
    Some(("logout", _)) => logout(config),
    Some(("session", _)) => session(config),
    _ => unreachable!(),
  }
  Ok(())
}
