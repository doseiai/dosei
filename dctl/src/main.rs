mod command;
mod config;
mod session;

mod cluster;
#[cfg(test)]
mod test;

use crate::command::certificate::new_certificate;
use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::new::new;
use crate::command::session::session;
use crate::command::token::list_token;
use crate::command::{certificate, env, token};
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
    .subcommand(env::sub_command())
    .subcommand(token::sub_command())
    .subcommand(certificate::sub_command())
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
      Some(("list", _)) => env::list_env(config),
      Some(("set", _)) => env::set_env(config),
      _ => unreachable!(),
    },
    Some(("certificate", params)) => match params.subcommand() {
      Some(("new", arg_matches)) => new_certificate(config, arg_matches),
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
