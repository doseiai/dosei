use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use ignore::WalkBuilder;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;

/// Create a *.tar.gz from the given path to a target path
///
/// # Arguments
///
/// * `output_path` - A string slice that points to where the output ?.tar.gz will be written
/// * `folder_path` - A Path slice that holds the path for the input directory
pub fn write_tar_gz(input_path: &Path, output_path: &str) -> Result<()> {
  let output_path = PathBuf::from(output_path);
  let tar_gz = File::create(&output_path)?;
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
