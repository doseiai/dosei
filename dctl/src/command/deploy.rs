use clap::{ArgMatches, Command};

pub fn sub_command() -> Command {
  Command::new("deploy").about("Deploy your app")
}

pub fn deploy(_: &ArgMatches) {
  todo!()
}
