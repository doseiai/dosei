use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/cron_job.proto", "src/cluster.proto"], &["src/"])?;
    Ok(())
}
