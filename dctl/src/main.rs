mod command;
mod config;
mod session;

mod cluster;
#[cfg(test)]
mod test;

use crate::command::certificate::new_certificate;
use crate::command::deploy::deploy;
use crate::command::dev::dev;
use crate::command::export::export;
use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::new::new;
use crate::command::run::run;
use crate::command::session::session;
use crate::command::token::list_token;
use crate::command::up::up;
use crate::command::{certificate, deploy, env, new, run, token, up};
use crate::config::{Config, VERSION};
use clap::Command;

fn cli() -> Command {
  Command::new("dctl")
    .version(VERSION)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(run::sub_command())
    .subcommand(up::sub_command())
    .subcommand(deploy::sub_command())
    .subcommand(Command::new("dev").about("Execute a Dosei App"))
    .subcommand(Command::new("export").about("Export a Dosei App"))
    .subcommand(Command::new("login").about("Log in to a cluster"))
    .subcommand(Command::new("logout").about("Log out from a cluster"))
    .subcommand(Command::new("session").about("Print active cluster session"))
    .subcommand(new::sub_command())
    .subcommand(env::sub_command())
    .subcommand(token::sub_command())
    .subcommand(certificate::sub_command())
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  let matches = cli().get_matches();
  match matches.subcommand() {
    Some(("run", arg_matches)) => run(arg_matches),
    Some(("dev", _)) => dev(),
    Some(("up", arg_matches)) => up(arg_matches),
    Some(("deploy", arg_matches)) => deploy(arg_matches),
    Some(("export", _)) => export(),
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
