#[allow(dead_code)]
pub fn command(_name: String) -> anyhow::Result<()> {
  // let path = ".env";
  // let file = File::open(path);
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
  // env_vars.remove(&name);
  //
  // let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
  //
  // for (key, value) in &env_vars {
  //   writeln!(file, "{}={}", key, value)?;
  // }

  Ok(())
}
