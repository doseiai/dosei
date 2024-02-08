use clap::{ArgMatches, Command};

pub fn sub_command() -> Command {
  Command::new("up").about("Create and start containers")
}

pub fn up(_: &ArgMatches) {
  todo!()
}
