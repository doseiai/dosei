use crate::command::find_and_print_dosei_config_extension;
use clap::{Arg, ArgMatches, Command};
use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;
use std::path::Path;

pub fn sub_command() -> Command {
  Command::new("up")
    .about("Create and start containers")
}

pub fn up(arg_matches: &ArgMatches) {
  todo!()
}
