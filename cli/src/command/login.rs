use crate::config::GITHUB_CLIENT_ID;

fn login() {
  let auth_url = format!("https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://localhost:8085/auth/github/cli&scope=read:user,user:email", GITHUB_CLIENT_ID);
}
