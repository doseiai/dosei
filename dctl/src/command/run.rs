use crate::command::find_and_print_dosei_config_extension;
use clap::{Arg, ArgMatches, Command};
use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;
use std::path::Path;

pub fn sub_command() -> Command {
  Command::new("run")
    .about("Execute a Dosei App")
    .arg(Arg::new("function").index(1).required(false))
}

pub fn run(arg_matches: &ArgMatches) {
  let function = arg_matches.get_one::<String>("function");
  let args = if function.is_some() {
    (function,)
  } else {
    (None,)
  };
  let path = Path::new(".");
  match find_and_print_dosei_config_extension(path) {
    Ok(extension) => match extension.as_str() {
      "py" => {
        Python::with_gil(|py| {
          let dosei_main = py.import("dosei_sdk.main").unwrap();
          if let Err(e) = dosei_main.call_method("run", args, None) {
            if e.is_instance(py, py.get_type::<PySystemExit>()) {
              if e.value(py).to_string() != "0" {
                println!("An error occurred: {:?}", e);
              }
            }
          }
        });
      }
      "js" | "mjs" | "cjs" | ".ts" | "tsx" => {
        todo!()
      }
      _ => unreachable!(),
    },
    Err(_) => {}
  };
}
