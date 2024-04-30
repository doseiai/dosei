pub(crate) mod set;
pub(crate) mod unset;

use clap::Command;

pub fn command() -> Command {
  Command::new("env")
    .about("Environment variables commands")
    .subcommand_required(true)
    .subcommand(set::command())
    .subcommand(unset::command())
}
