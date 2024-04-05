mod command;
mod config;
mod session;

mod cluster;
mod git;
#[cfg(test)]
mod test;
mod util;

use crate::command::certificate::new_certificate;
use crate::command::deploy::deploy;
use crate::command::login::login;
use crate::command::logout::logout;
use crate::command::new::new;
use crate::command::run::run;
use crate::command::service::list_services;
use crate::command::session::session;
use crate::command::token::list_token;
use crate::command::{certificate, deploy, env, info, new, run, service, token};
use crate::config::{Config, VERSION};
use clap::Command;

fn cli() -> Command {
  Command::new("dosei")
    .version(VERSION)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(run::sub_command())
    .subcommand(service::sub_command())
    .subcommand(new::sub_command())
    .subcommand(deploy::sub_command())
    .subcommand(Command::new("login").about("Log in to a cluster"))
    .subcommand(Command::new("logout").about("Log out from a cluster"))
    .subcommand(Command::new("session").about("Print active cluster session"))
    .subcommand(env::sub_command())
    .subcommand(Command::new("info").about("Print cluster information."))
    .subcommand(token::sub_command())
    .subcommand(certificate::sub_command())
}

fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  match cli().get_matches().subcommand() {
    Some(("run", arg_matches)) => run(arg_matches),
    Some(("deploy", _)) => deploy(config)?,
    Some(("login", _)) => login(config),
    Some(("info", _)) => info::cluster_info(config),
    Some(("logout", _)) => logout(config),
    Some(("service", params)) => match params.subcommand() {
      Some(("list", _)) => list_services(config),
      _ => unreachable!(),
    },
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
