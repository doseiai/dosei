#[allow(dead_code)]
pub fn command(_name: String, _arg_value: Option<String>) -> anyhow::Result<()> {
  // let mut value = String::new();
  // if dosei_util::secret::is_secret_env(&name) {
  //   value = rpassword::prompt_password("Enter the secret value: ")?;
  //
  //   let login_url = format!("{}/secret", config.api_base_url);
  //   let body = json!({ "name": name.clone(), "value": value.clone() });
  //   let response = Client::new()
  //     .post(login_url)
  //     .json(&body)
  //     .bearer_auth(config.bearer_ssh_token(None)?)
  //     .send()?;
  //   let status_code = response.status();
  //   if status_code.is_success() {
  //     let secret = response.json::<Value>()?;
  //     value = secret.get("value").unwrap().to_string();
  //   } else {
  //     response.error_for_status()?;
  //   }
  // } else if let Some(input_value) = arg_value {
  //   value = input_value.to_string()
  // } else {
  //   print!("Enter the environment variable value: ");
  //   io::stdout().flush()?;
  //   io::stdin().read_line(&mut value)?;
  //   value = value.trim().to_string();
  // }
  //
  // let path = ".env";
  // let file = File::open(path);
  //
  // let mut env_vars = BTreeMap::new();
  //
  // if let Ok(file) = file {
  //   let reader = BufReader::new(file);
  //   for line in reader.lines() {
  //     let line = line?;
  //     if let Some((key, value)) = line.split_once('=') {
  //       env_vars.insert(key.trim().to_string(), value.trim().to_string());
  //     }
  //   }
  // }
  //
  // env_vars.insert(name.to_string(), value.to_string());
  //
  // let mut file = OpenOptions::new()
  //   .write(true)
  //   .create(true)
  //   .truncate(true)
  //   .open(path)?;
  //
  // for (key, value) in env_vars {
  //   writeln!(file, "{}={}", key, value)?;
  // }

  Ok(())
}
