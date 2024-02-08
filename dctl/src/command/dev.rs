use crate::command::find_and_print_dosei_config_extension;
use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;
use std::path::Path;

pub fn dev() {
  let path = Path::new(".");
  match find_and_print_dosei_config_extension(path) {
    Ok(extension) => match extension.as_str() {
      "py" => {
        Python::with_gil(|py| {
          let dosei_main = py.import("dosei.main").unwrap();
          let result = dosei_main.call_method("dev", (), None);
          if let Err(e) = result {
            if e.is_instance(py, py.get_type::<PySystemExit>()) && e.value(py).to_string() != "0" {
              println!("An error occurred: {:?}", e);
            }
          }
        });
      }
      _ => unreachable!(),
    },
    Err(_) => {}
  };
}
