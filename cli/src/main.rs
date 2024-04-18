mod command;
mod config;

use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::whoami::whoami;
use crate::command::{login, logout, whoami};
use crate::config::Config;
use clap::Command;

fn cli() -> Command {
  Command::new("dosei")
    .version(env!("CARGO_PKG_VERSION"))
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(login::command())
    .subcommand(logout::command())
    .subcommand(whoami::command())
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  match cli().get_matches().subcommand() {
    Some(("login", _)) => login(config)?,
    Some(("logout", _)) => logout(config)?,
    Some(("whoami", _)) => whoami(config)?,
    _ => unreachable!(),
  };
  Ok(())
}
