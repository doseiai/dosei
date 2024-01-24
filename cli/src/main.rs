mod command;
mod config;
mod session;

#[cfg(test)]
mod test;

use crate::command::env::{list_env, set_env};
use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::session::session;
use crate::config::{Config, VERSION};
use clap::{ArgMatches, Command};

fn cli() -> Command {
  let env_command = Command::new("env")
    .about("Environment variables commands")
    .subcommand_required(true)
    .subcommand(Command::new("list").about("List environment variables"))
    .subcommand(Command::new("set").about("Set environment variables"));

  Command::new("dctl")
    .version(VERSION)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(Command::new("login").about("Log in to a cluster"))
    .subcommand(Command::new("logout").about("Log out from a cluster"))
    .subcommand(Command::new("session").about("Print active cluster session"))
    .subcommand(env_command)
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  let matches = cli().get_matches();
  match matches.subcommand() {
    Some(("login", _)) => login(config),
    Some(("logout", _)) => logout(config),
    Some(("session", _)) => session(config),
    Some(("env", params)) => match params.subcommand() {
      Some(("list", _)) => list_env(config),
      Some(("set", _)) => set_env(config),
      _ => unreachable!(),
    },
    _ => unreachable!(),
  }
  Ok(())
}
