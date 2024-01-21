use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserGithub {
  login: String,
  id: i64,
  access_token: Option<String>,
  emails: Vec<UserGithubEmail>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserGithubEmail {
  email: String,
  primary: bool,
  verified: bool,
  visibility: Option<String>,
}
