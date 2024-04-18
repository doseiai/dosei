mod command;
mod config;

use crate::command::login::login;
use crate::command::logout::logout;
use crate::config::Config;
use clap::Command;

fn cli() -> Command {
  Command::new("dosei")
    .version(env!("CARGO_PKG_VERSION"))
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(Command::new("login").about("Log in to a cluster"))
    .subcommand(Command::new("logout").about("Log out from a cluster"))
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  match cli().get_matches().subcommand() {
    Some(("login", _)) => login(config)?,
    Some(("logout", _)) => logout(config)?,
    _ => unreachable!(),
  };
  Ok(())
}
