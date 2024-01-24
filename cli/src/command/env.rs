use crate::config::Config;
use crate::session::get_session_user;

pub fn list_env(config: &'static Config) {
  let user = match get_session_user(config) {
    Ok(user) => user,
    _ => panic!("Something went wrong"),
  };
  println!("{:?}", user);
}

pub fn set_env(config: &'static Config) {
  todo!();
}
