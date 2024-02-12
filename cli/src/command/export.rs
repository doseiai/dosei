use crate::command::find_and_print_dosei_config_extension;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn export() {
  let path = Path::new(".");
  if let Ok(extension) = find_and_print_dosei_config_extension(path) {
    match extension.as_str() {
      "py" => {
        if let Err(err) = Command::new("python3")
          .arg("-c")
          .arg("from dosei_sdk import main\nmain.export()")
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .output()
        {
          eprintln!("{:?}", err);
        };
      }
      "js" | "mjs" | "cjs" | ".ts" | "tsx" => {
        let node_command = r#"
        (async () => {
          const { export_config } = await import('@dosei/dosei');
          await export_config();
        })();"#;
        if let Err(err) = Command::new("node")
          .arg("-e")
          .arg(node_command)
          .env("NODE_PATH", ".")
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .output()
        {
          eprintln!("{:?}", err);
        };
      }
      _ => unreachable!(),
    }
  }
}
