use crate::command::find_and_print_dosei_config_extension;
use pyo3::exceptions::PySystemExit;
use pyo3::prelude::*;
use std::path::Path;

pub fn export() {
  let path = Path::new(".");
  match find_and_print_dosei_config_extension(path) {
    Ok(extension) => match extension.as_str() {
      "py" => {
        Python::with_gil(|py| {
          let dosei_main = py.import("dosei.main").unwrap();
          let result = dosei_main.call_method("export", (), None);
          if let Err(e) = result {
            if e.is_instance(py, py.get_type::<PySystemExit>()) {
              if e.value(py).to_string() != "0" {
                println!("An error occurred: {:?}", e);
              }
            }
          }
        });
      }
      "js" | "mjs" | "cjs" | ".ts" | "tsx" => {
        use std::process::{Command, Stdio};
        let node_command = r#"
        (async () => {
          const { export_config } = await import('@dosei/dosei');
          await export_config();
        })();"#;
        let output = Command::new("node")
          .arg("-e")
          .arg(node_command)
          .env("NODE_PATH", ".")
          .stdout(Stdio::piped())
          .stderr(Stdio::piped())
          .output();
        println!("{:?}", output);
      }
      _ => unreachable!(),
    },
    Err(_) => {}
  };
}
