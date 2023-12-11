use std::io::Result;

fn main() -> Result<()> {
  prost_build::compile_protos(&["src/cluster.proto", "src/cron_job.proto"], &["src/"])?;
  Ok(())
}
