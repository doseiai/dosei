use crate::config::Config;
use crate::util::write_tar_gz;
use clap::Command;
use std::env;

pub fn sub_command() -> Command {
  Command::new("deploy").about("Deploy Dosei App")
}

pub fn deploy(config: &'static Config) {
  println!("{:?}", env::current_dir());
  let path = env::current_dir().expect("Something went wrong");
  write_tar_gz(path.as_path(), ".dosei/output.tar.gz").unwrap();

  let form = reqwest::blocking::multipart::Form::new()
    .file("file", ".dosei/output.tar.gz")
    .expect("failed");

  config
    .cluster_api_client()
    .expect("Client connection failed")
    .post(format!("{}/deploy", config.api_base_url))
    .multipart(form)
    .send()
    .unwrap();
  println!("Do deploy thing")
}
