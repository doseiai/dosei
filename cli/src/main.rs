mod command;
mod config;

use crate::command::deploy::deploy;
use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::new::new;
use crate::command::whoami::whoami;
use crate::command::{deploy, env, login, logout, new, whoami};
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
    .subcommand(deploy::command())
    .subcommand(new::command())
    .subcommand(env::command())
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  match cli().get_matches().subcommand() {
    Some(("login", arg_matches)) => login(arg_matches, config)?,
    Some(("logout", _)) => logout(config)?,
    Some(("whoami", _)) => whoami(config)?,
    Some(("new", arg_matches)) => new(arg_matches)?,
    Some(("env", params)) => match params.subcommand() {
      Some(("set", arg_matches)) => env::set::set_env(arg_matches, config)?,
      Some(("unset", arg_matches)) => env::unset::unset_env(arg_matches, config)?,
      _ => unreachable!(),
    },
    Some(("deploy", _)) => deploy(config)?,
    _ => unreachable!(),
  };
  Ok(())
}
