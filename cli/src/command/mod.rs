use std::fs;
use std::path::Path;

pub(crate) mod certificate;
pub(crate) mod deploy;
pub(crate) mod env;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod new;
pub(crate) mod run;
pub(crate) mod service;
pub(crate) mod session;
pub(crate) mod token;

fn find_and_print_dosei_config_extension(directory: &Path) -> anyhow::Result<String> {
  if let Ok(entries) = fs::read_dir(directory) {
    for entry in entries.filter_map(Result::ok) {
      let path = entry.path();
      if path.is_file() {
        if let Some(stem) = path.file_stem() {
          if stem.to_string_lossy().eq("dosei") {
            if let Some(extension) = path.extension() {
              return Ok(extension.to_string_lossy().to_string());
            }
          }
        }
      }
    }
  }
  Err(anyhow::Error::msg("No 'dosei.*' file found."))
}
