use crate::config::Config;
use clap::Command;

pub fn sub_command() -> Command {
  Command::new("deploy").about("Deploy commands")
}

pub fn deploy(config: &'static Config) {
  println!("Deploy!");
}
