const SECRET_PREFIX: &str = "DOSEI_SECRET_";

pub fn is_secret_env(name: &str) -> bool {
  name.starts_with(SECRET_PREFIX) && name.len() > SECRET_PREFIX.len()
}
