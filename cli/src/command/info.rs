use crate::cluster::get_cluster_info;
use crate::config::Config;

pub fn cluster_info(config: &'static Config) {
  let cluster_info = get_cluster_info(config).unwrap();
  let json_info = serde_json::to_string(&cluster_info).unwrap();
  println!("{}", json_info);
}
