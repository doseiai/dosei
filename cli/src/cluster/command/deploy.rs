use crate::cluster::{CliClusterInit, Cluster};
use crate::config::{ClusterConfig, Config};
use crate::file::expand_tilde;
use crate::ssh::SSH;
use anyhow::{anyhow, Context};
use colored::Colorize;
use dosei_schema::cluster::ClusterInit;
use ssh2::Session;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::{fs, io};

pub fn command(allow_invalid_domain: bool) -> anyhow::Result<()> {
  let cluster = Cluster::get_from_dosei_file()?;

  if !ClusterInit::validate_domain(&cluster.name) {
    eprintln!(
      "{}",
      format!(
        "WARNING: Cluster name '{}' is not a fully qualified domain name.",
        cluster.name
      )
      .yellow()
    );
    if !allow_invalid_domain {
      // Prompt user for confirmation
      eprint!("Do you want to continue anyway? [y/N]: ");
      io::stdout().flush().unwrap();

      let mut input = String::new();
      io::stdin().read_line(&mut input).unwrap();

      if !input.trim().eq_ignore_ascii_case("y") {
        return Err(anyhow!("Deployment cancelled due to invalid domain name"));
      }
    }
  }

  let key_path_or_content = if let Some(identity) = &cluster.identity {
    identity.clone()
  } else if let Ok(key) = std::env::var("DOSEI_SSH_KEY") {
    key
  } else {
    SSH::get_default_ssh_key_path()
      .context("Failed to get default ssh key path. Define one")?
      .to_string_lossy()
      .to_string()
  };

  // Create and serialize the ClusterInit object
  let cluster_init = CliClusterInit(ClusterInit {
    name: cluster.name.clone(),
    dosei_public_key: SSH::generate_ed25519_key()?,
    accounts: cluster.accounts,
  });

  // Collect all servers. If none specified, use the cluster name as the single server.
  let servers: Vec<String> = if let Some(servers) = &cluster.servers {
    if servers.is_empty() {
      vec![cluster.name.clone()]
    } else {
      servers.clone()
    }
  } else {
    vec![cluster.name.clone()]
  };

  // First server is main, rest are workers
  let main_server_url = &servers[0];
  let (main_username, main_hostname) = parse_server_url(main_server_url)?;

  // --- Deploy main node ---
  println!("🔐 Connecting to main node {}...", main_server_url);
  let lock_sess = connect_ssh(&main_hostname, &main_username, &key_path_or_content)?;

  cluster_init.create_lock(&lock_sess)?;
  let _lock_guard = scopeguard::guard((), |_| {
    if let Err(e) = cluster_init.remove_lock(&lock_sess) {
      eprintln!("Warning: Failed to remove lock file: {}", e);
    }
  });

  cluster_init.save_to_cluster(&lock_sess)?;
  cluster_init.install_docker_on_remote(&lock_sess, &main_username)?;

  // Reconnect SSH so docker group membership takes effect
  println!("🔐 Reconnecting to main node {}...", main_server_url);
  let main_sess = connect_ssh(&main_hostname, &main_username, &key_path_or_content)?;

  println!("\n Starting doseid (main mode) on {}", main_hostname);
  cluster_init.run_doseid_container(&main_sess, None)?;

  // --- Deploy worker nodes ---
  if servers.len() > 1 {
    // Detect main node's internal IP for inter-node communication.
    // Workers use this instead of the public IP so all traffic stays on the private network.
    let main_internal_ip = detect_internal_ip(&main_sess)?;
    println!("Main node internal IP: {}", main_internal_ip);
    let main_url = format!("http://{}:8080", main_internal_ip);

    for server_url in &servers[1..] {
      let (username, hostname) = parse_server_url(server_url)?;

      println!("\n Connecting to worker node {}...", server_url);
      let sess = connect_ssh(&hostname, &username, &key_path_or_content)?;

      cluster_init.save_to_cluster(&sess)?;
      cluster_init.install_docker_on_remote(&sess, &username)?;

      // Reconnect SSH so docker group membership takes effect
      println!("🔐 Reconnecting to worker node {}...", server_url);
      let sess = connect_ssh(&hostname, &username, &key_path_or_content)?;

      println!("Starting doseid (worker mode) on {}", hostname);
      cluster_init.run_doseid_container(&sess, Some(&main_url))?;
    }

    println!(
      "\nCluster deployed: 1 main + {} worker(s)",
      servers.len() - 1
    );
  }

  // Save config only if cluster is not already configured
  let mut user_config = Config::load()?;
  if user_config.get_cluster(&cluster.name).is_none() {
    let current_dir = std::env::current_dir()?;
    let dosei_dir = current_dir.join(".dosei");
    let private_key_path = dosei_dir.join("dosei_ed25519");
    user_config.add_cluster(
      cluster.name.clone(),
      ClusterConfig {
        id: None,
        username: String::from("dosei"),
        ssh_key: private_key_path.to_str().map(|s| s.to_string()),
      },
    );
    user_config.save()?;
  }

  Ok(())
}

/// Parse "user@hostname" or just "hostname" into (username, hostname).
fn parse_server_url(server_url: &str) -> anyhow::Result<(String, String)> {
  let server_url = if !server_url.contains('@') {
    format!("root@{}", server_url)
  } else {
    server_url.to_string()
  };

  let parts: Vec<&str> = server_url.split('@').collect();
  if parts.len() != 2 {
    return Err(anyhow!("Invalid server URL format. Expected user@hostname"));
  }
  Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Detect the internal/private IP of a remote machine via SSH.
/// Tries common methods to find a private network IP (10.x, 172.16-31.x, 192.168.x).
fn detect_internal_ip(session: &Session) -> anyhow::Result<String> {
  // Try hostname -I first (space-separated list of all IPs)
  let (exit_code, output) = SSH::execute_command(session, "hostname -I")?;
  if exit_code == 0 {
    for ip in output.split_whitespace() {
      if is_private_ip(ip) {
        return Ok(ip.to_string());
      }
    }
    // If no private IP found, use the first IP
    if let Some(first_ip) = output.split_whitespace().next() {
      return Ok(first_ip.to_string());
    }
  }

  // Fallback: use ip route to find the default source IP
  let (exit_code, output) =
    SSH::execute_command(session, "ip route get 1.1.1.1 | awk '{print $7; exit}'")?;
  if exit_code == 0 && !output.trim().is_empty() {
    return Ok(output.trim().to_string());
  }

  Err(anyhow!("Could not detect internal IP of remote server"))
}

/// Check if an IP string is in a private range (RFC 1918).
fn is_private_ip(ip: &str) -> bool {
  if let Ok(addr) = ip.parse::<std::net::Ipv4Addr>() {
    let octets = addr.octets();
    matches!(
      octets,
      [10, ..] | [172, 16..=31, ..] | [192, 168, ..]
    )
  } else {
    false
  }
}

/// Establish an SSH session to hostname:22 with the given key.
fn connect_ssh(hostname: &str, username: &str, key_path_or_content: &str) -> anyhow::Result<Session> {
  let tcp =
    TcpStream::connect(format!("{}:22", hostname)).context("Failed to connect to the server")?;

  let mut sess = Session::new().context("Failed to create SSH session")?;
  sess.set_tcp_stream(tcp);
  sess.handshake().context("SSH handshake failed")?;

  if key_path_or_content.contains("-----BEGIN") {
    let temp_file = tempfile::NamedTempFile::new().context("Failed to create temporary file")?;
    fs::write(&temp_file, key_path_or_content).context("Failed to write key to temp file")?;
    sess
      .userauth_pubkey_file(username, None, temp_file.path(), None)
      .context("Authentication failed")?;
  } else {
    let expanded_path = expand_tilde(key_path_or_content);
    let private_key_path = Path::new(&expanded_path);
    sess
      .userauth_pubkey_file(username, None, private_key_path, None)
      .context("Authentication failed")?;
  }

  Ok(sess)
}
