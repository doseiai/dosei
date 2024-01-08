use crate::config::Config;
use once_cell::sync::Lazy;

#[cfg(test)]
pub(crate) static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().unwrap());
