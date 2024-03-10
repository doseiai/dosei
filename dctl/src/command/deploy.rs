use crate::config::Config;
use crate::util::write_tar_gz;
use clap::Command;
use std::env;
use std::fs::create_dir_all;

pub fn sub_command() -> Command {
  Command::new("deploy").about("Deploy Dosei App")
}

pub fn deploy(config: &'static Config) {
  println!("{:?}", env::current_dir());
  let path = env::current_dir().expect("Something went wrong");

  let mut dst_path = path.clone();
  dst_path.push(".dosei/output.tar.gz");
  if let Some(parent_dir) = dst_path.parent() {
    println!("{:?}", parent_dir);
    create_dir_all(parent_dir).unwrap();
  }

  write_tar_gz(&path, &dst_path).unwrap();

  let form = reqwest::blocking::multipart::Form::new()
    .file("file", dst_path)
    .expect("failed");

  config
    .cluster_api_client()
    .expect("Client connection failed")
    .post(format!("{}/deploy", config.api_base_url))
    .multipart(form)
    .bearer_auth(config.bearer_token())
    .send()
    .unwrap();
  println!("Do deploy thing")
}
