use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserGithub {
  login: String,
  id: i64,
  email: String,
  pub emails: Option<Vec<UserGithubEmail>>,
  pub access_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserGithubEmail {
  pub email: String,
  pub primary: bool,
  pub verified: bool,
  pub visibility: Option<String>,
}
