pub mod git;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use ignore::WalkBuilder;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use tokio::task;

/// Create a *.tar.gz from the given path to a target path
///
pub fn write_tar_gz(input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
  let output_path = PathBuf::from(output_path);
  let tar_gz = File::create(output_path)?;
  let enc = GzEncoder::new(tar_gz, Compression::default());
  let mut tar = Builder::new(enc);

  let walker = WalkBuilder::new(input_path)
    .ignore(true)
    .git_ignore(true)
    .git_exclude(true)
    .build();

  for result in walker {
    let entry = result?;
    let path = entry.path();
    if path.is_dir() {
      continue;
    }

    if let Ok(stripped_path) = path.strip_prefix(input_path) {
      tar.append_path_with_name(path, stripped_path)?;
    }
  }

  tar.into_inner()?.finish()?;
  Ok(())
}

pub async fn extract_tar_gz_from_memory(
  combined_data: &[u8],
  target_folder: &Path,
) -> anyhow::Result<()> {
  let combined_data_owned = combined_data.to_owned();
  let target_folder_buf = target_folder.to_path_buf();
  task::spawn_blocking(move || {
    let cursor = Cursor::new(combined_data_owned);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);
    archive.unpack(target_folder_buf)?;
    Ok(())
  })
  .await?
}

pub fn dosei_service_config(directory: &Path) -> anyhow::Result<DoseiConfig> {
  if let Ok(entries) = fs::read_dir(directory) {
    for entry in entries.filter_map(Result::ok) {
      let path = entry.path();
      if path.is_file() {
        if let Some(stem) = path.file_stem() {
          if stem.to_string_lossy().eq("dosei") {
            if let Some(extension) = path.extension() {
              return Ok(DoseiConfig {
                path: path.clone(),
                extension: extension.to_string_lossy().to_string(),
              });
            }
          }
        }
      }
    }
  }
  Err(anyhow::Error::msg("No 'dosei.*' file found."))
}

pub struct DoseiConfig {
  pub path: PathBuf,
  pub extension: String,
}
