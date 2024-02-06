use std::fs;
use std::path::Path;

pub(crate) mod certificate;
pub(crate) mod dev;
pub(crate) mod env;
pub(crate) mod export;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod new;
pub(crate) mod run;
pub(crate) mod session;
pub(crate) mod token;
pub(crate) mod up;

fn find_and_print_dosei_config_extension(directory: &Path) -> anyhow::Result<String> {
  if let Ok(entries) = fs::read_dir(directory) {
    for entry in entries.filter_map(Result::ok) {
      let path = entry.path();
      if path.is_file() {
        if let Some(stem) = path.file_stem() {
          if stem.to_string_lossy().eq("dosei_config") {
            if let Some(extension) = path.extension() {
              return Ok(extension.to_string_lossy().to_string());
            }
          }
        }
      }
    }
  }
  Err(anyhow::Error::msg("No 'dosei_config.*' file found."))
}
