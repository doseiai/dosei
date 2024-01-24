mod command;
mod config;
mod session;

#[cfg(test)]
mod test;

use crate::command::env::{list_env, set_env};
use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::new::new;
use crate::command::session::session;
use crate::command::token::list_token;
use crate::config::{Config, VERSION};
use clap::{Arg, Command};

fn cli() -> Command {
  let env_subcommand = Command::new("env")
    .about("Environment variables commands")
    .subcommand_required(true)
    .subcommand(Command::new("list").about("List environment variables"))
    .subcommand(Command::new("set").about("Set environment variables"));

  let token_subcommand = Command::new("token")
    .about("Tokens commands")
    .subcommand_required(true)
    .subcommand(Command::new("list").about("List tokens"));

  let new_subcommand = Command::new("new")
    .about("New resource commands")
    .arg(
      Arg::new("template")
        .short('t')
        .long("template")
        .value_parser(["fastapi"])
        .default_value("fastapi"),
    )
    .arg(Arg::new("name").index(1).required(true));

  Command::new("dosei")
    .version(VERSION)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(Command::new("login").about("Log in to a cluster"))
    .subcommand(Command::new("logout").about("Log out from a cluster"))
    .subcommand(Command::new("session").about("Print active cluster session"))
    .subcommand(env_subcommand)
    .subcommand(token_subcommand)
    .subcommand(new_subcommand)
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  let matches = cli().get_matches();
  match matches.subcommand() {
    Some(("login", _)) => login(config),
    Some(("logout", _)) => logout(config),
    Some(("session", _)) => session(config),
    Some(("new", arg_matches)) => new(config, arg_matches),
    Some(("env", params)) => match params.subcommand() {
      Some(("list", _)) => list_env(config),
      Some(("set", _)) => set_env(config),
      _ => unreachable!(),
    },
    Some(("token", params)) => match params.subcommand() {
      Some(("list", _)) => list_token(config),
      _ => unreachable!(),
    },
    _ => unreachable!(),
  }
  Ok(())
}
