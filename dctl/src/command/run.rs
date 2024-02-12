use crate::command::find_and_print_dosei_config_extension;
use clap::{Arg, ArgMatches, Command};
use std::path::Path;
use std::process::Stdio;

pub fn sub_command() -> Command {
  Command::new("run")
    .about("Execute a Dosei App")
    .arg(Arg::new("function").index(1).required(false))
}

pub fn run(arg_matches: &ArgMatches) {
  let function = arg_matches.get_one::<String>("function");
  let path = Path::new(".");
  if let Ok(extension) = find_and_print_dosei_config_extension(path) {
    match extension.as_str() {
      "py" => {
        let arg = match function {
          Some(command) => format!("from dosei_sdk import main\nmain.run(\"{}\")", command),
          None => "from dosei_sdk import main\nmain.run()".to_string(),
        };
        if let Err(err) = std::process::Command::new("python3")
          .arg("-c")
          .arg(arg)
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .output()
        {
          eprintln!("{:?}", err);
        };
      }
      "js" | "mjs" | "cjs" | ".ts" | "tsx" => {
        todo!()
      }
      _ => unreachable!(),
    }
  }
}
