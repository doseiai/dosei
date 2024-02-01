use crate::cluster::get_cluster_info;
use crate::config::{Config, SessionCredentials};
use clap::Command;
use reqwest::Url;
use tiny_http::{Header, Response, Server};

pub fn sub_command() -> Command {
  Command::new("login").about("Log in to a cluster")
}

pub fn login(config: &'static Config) {
  let host = "localhost:8085";
  let base_url = format!("http://{}", host);
  let server = Server::http(host).unwrap();
  let cluster_info = get_cluster_info(config).expect("Couldn't fetch cluster info");
  let auth_url = format!(
    "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}/auth/github/cli&scope=read:user,user:email",
    cluster_info.server.integration.github.expect("Github Integration disabled").client_id,
    base_url
  );

  if webbrowser::open(&auth_url).is_ok() {
    println!("Your browser has been opened to visit:\n\n{}\n", auth_url);
  }
  if let Some(request) = server.incoming_requests().next() {
    if request.url().starts_with("/auth/github/cli") {
      let url = Url::parse(&format!("{}{}", base_url, request.url())).unwrap();
      let mut code = String::new();
      for (key, value) in url.query_pairs() {
        if key == "code" {
          code = value.into_owned();
          break;
        }
      }
      let response = config
        .cluster_api_client()
        .expect("Client connection failed")
        .get(format!("{}/auth/github/cli", config.api_base_url))
        .query(&[("code", code)])
        .send()
        .unwrap();
      let status_code = response.status();
      if status_code.is_success() {
        let session = response.json::<SessionCredentials>().unwrap();
        config.store_token_from_session(&session).unwrap();
        request
          .respond(Response::empty(302).with_header(
            Header::from_bytes(&b"Location"[..], &b"https://dosei.ai/login/cli"[..]).unwrap(),
          ))
          .unwrap();
        return println!("Login Succeeded!");
      }
    }
    let error_message = "Login Failed!";
    request
      .respond(Response::from_string(error_message).with_status_code(400))
      .unwrap();
    eprintln!("{}", error_message);
  }
}
