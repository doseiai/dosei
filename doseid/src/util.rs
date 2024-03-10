use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use tar::{Archive, Builder};
use tokio::task;

pub(crate) async fn write_tar_gz(output_path: &str, folder_path: &Path) -> anyhow::Result<()> {
  let output_path = output_path.to_owned();
  let folder_path = folder_path.to_path_buf();
  task::spawn_blocking(move || {
    let tar_gz = File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", folder_path)?;

    tar.into_inner()?.finish()?;
    Ok(())
  })
  .await?
}

pub(crate) async fn extract_tar_gz_from_memory(
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

pub(crate) async fn read_tar_gz_content(output_path: &str) -> Vec<u8> {
  use tokio::fs::File;
  use tokio::io::AsyncReadExt;

  let mut file = File::open(output_path).await.unwrap();
  let mut contents = Vec::new();
  file.read_to_end(&mut contents).await.unwrap();
  contents
}
