use std::fs;
use std::io::Result;

fn main() -> Result<()> {
  let proto_files: Vec<_> = fs::read_dir("schema")?
    .filter_map(Result::ok)
    .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("proto"))
    .map(|entry| entry.path())
    .filter_map(|path| path.to_str().map(String::from))
    .collect();

  prost_build::compile_protos(&proto_files, &["schema/"])?;
  Ok(())
}
