use crate::ssh::SSH;
use anyhow::{anyhow, Context};
use dosei_schema::cluster::{
  ClusterAccount, ClusterInit, REMOTE_CLUSTER_DAEMON_FOLDER, REMOTE_CLUSTER_DEPLOY_LOCK_FILE,
  REMOTE_CLUSTER_DOSEI_FOLDER, REMOTE_CLUSTER_INIT_FILE, REMOTE_CLUSTER_POSTGRES_VOLUME,
};
use dosei_schema::DoseiObject;
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::fs;
use std::ops::Deref;
use std::path::Path;

pub(crate) mod command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cluster {
  pub name: String,
  pub servers: Option<Vec<String>>,
  pub identity: Option<String>,
  pub accounts: Option<Vec<ClusterAccount>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliClusterInit(pub ClusterInit);

impl Deref for CliClusterInit {
  type Target = ClusterInit;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Cluster {
  pub fn from_json_file() -> anyhow::Result<Self> {
    let cluster_path = Path::new(".").join(ClusterInit::json_path());
    let app_data = fs::read_to_string(&cluster_path)
      .context(format!("Failed to read cluster file at {:?}", cluster_path))?;
    Ok(serde_json::from_str::<Self>(&app_data)?)
  }
}

impl CliClusterInit {
  pub fn create_lock(&self, session: &Session) -> anyhow::Result<()> {
    // First ensure ~/.dosei directory exists
    SSH::execute_command(
      session,
      &format!("mkdir -p {}", REMOTE_CLUSTER_DOSEI_FOLDER),
    )?;

    // Check if lock file exists
    let remote_lock = SSH::check_file_exists(session, REMOTE_CLUSTER_DEPLOY_LOCK_FILE)?;
    if remote_lock {
      return Err(anyhow!("Another cluster deployment is in progress. If you're sure no other process is running, remove ~/.dosei/deploy.lock manually."));
    }
    // Create the lock file
    SSH::execute_command(
      session,
      &format!("touch {}", REMOTE_CLUSTER_DEPLOY_LOCK_FILE),
    )?;

    Ok(())
  }

  /// Run the doseid container on a remote server.
  /// If `main_url` is None, runs in main mode (with Postgres volume).
  /// If `main_url` is Some, runs in worker mode pointing to the main node.
  pub fn run_doseid_container(
    &self,
    session: &Session,
    main_url: Option<&str>,
  ) -> anyhow::Result<()> {
    // First ensure ~/.dosei directory exists
    SSH::execute_command(
      session,
      &format!("mkdir -p {}", REMOTE_CLUSTER_DOSEI_FOLDER),
    )?;

    // Define container name
    let container_name = "doseid";

    // Check and stop existing container if needed
    let check_running_command = format!("docker ps -q -f name={}", container_name);
    let running_result = SSH::execute_command(session, &check_running_command);
    if let Ok(output) = running_result {
      if !output.1.trim().is_empty() {
        // Container is running, let's stop it
        println!(
          "Container '{}' is already running. Stopping it to continue.",
          container_name
        );

        let stop_command = format!("docker stop {container_name}");
        let stop_result = SSH::execute_command(session, &stop_command);
        if stop_result.is_err() {
          return Err(anyhow!(
            "Failed to stop existing container '{}'. Please stop it manually.",
            container_name
          ));
        }
      }
    }

    // Clean up old container
    let rm_command = format!("docker rm {container_name} 2>/dev/null || true");
    SSH::execute_command(session, &rm_command)?;

    // Allow overriding the docker image via env var (useful for testing/CI)
    let docker_image = std::env::var("DOSEID_IMAGE").unwrap_or_else(|_| {
      let image_version = env!("CARGO_PKG_VERSION");
      format!("doseidotio/doseid:{}", image_version)
    });

    // Optional: forward DASHBOARD_IMAGE into the container
    let dashboard_image_env = std::env::var("DASHBOARD_IMAGE")
      .map(|img| format!("-e DASHBOARD_IMAGE={} ", img))
      .unwrap_or_default();

    // Pull the image
    println!("Pulling image {}...", docker_image);
    let pull_cmd = format!("docker pull {}", docker_image);
    let pull_result = SSH::execute_command(session, &pull_cmd)?;
    if pull_result.0 != 0 {
      return Err(anyhow!("Failed to pull image {}: {}", docker_image, pull_result.1));
    }

    // Build the docker run command based on mode
    let docker_command = match main_url {
      None => {
        // Main mode: runs Postgres, mounts Docker socket + data volumes
        format!(
          "docker run -d \
            --network host \
            -v /var/run/docker.sock:/var/run/docker.sock \
            -v {}:/var/lib/doseid \
            -v {}:/var/lib/postgresql/17/main \
            --name {} {}",
          REMOTE_CLUSTER_DAEMON_FOLDER,
          REMOTE_CLUSTER_POSTGRES_VOLUME,
          container_name,
          docker_image
        )
      }
      Some(url) => {
        // Worker mode: no Postgres, communicates with main node over HTTP
        format!(
          "docker run -d \
            --network host \
            -v /var/run/docker.sock:/var/run/docker.sock \
            -v {}:/var/lib/doseid \
            -e DOSEID_MAIN_URL={} \
            --name {} {}",
          REMOTE_CLUSTER_DAEMON_FOLDER,
          url,
          container_name,
          docker_image
        )
      }
    };

    // Execute the docker run command
    let docker_run = SSH::execute_command(session, &docker_command)?;
    if docker_run.0 != 0 {
      return Err(anyhow!(format!(
        "Failed to start DoseiD Container. {}",
        docker_run.1
      )));
    }
    println!("Running DoseiD container: {}", docker_run.1.trim());

    Ok(())
  }

  pub fn remove_lock(&self, session: &Session) -> anyhow::Result<()> {
    SSH::execute_command(
      session,
      &format!("rm -f {}", REMOTE_CLUSTER_DEPLOY_LOCK_FILE),
    )?;
    Ok(())
  }

  pub fn install_docker_on_remote(&self, session: &Session, username: &str) -> anyhow::Result<()> {
    println!("\n🐳 Docker Check:");
    // Check for docker binary
    let docker_exists = SSH::check_file_exists(session, "/usr/bin/docker")?
      || SSH::check_file_exists(session, "/bin/docker")?;

    if !docker_exists {
      println!("Docker is NOT installed on the remote server");

      println!("\nInstalling Docker...");
      if let Err(e) = self._install_docker(session, username) {
        println!("Failed to install Docker: {}", e);
        return Err(e);
      }
      println!("Docker successfully installed!");
    } else {
      // Try to get Docker version
      let (exit_code, docker_version) = SSH::execute_command(session, "docker --version")?;

      if exit_code == 0 && !docker_version.is_empty() {
        println!("Docker is installed on the remote server:");
        println!("{}", docker_version.trim());
      } else {
        println!("Docker binary found but failed to get version information");

        println!("\nReinstalling Docker...");
        if let Err(e) = self._install_docker(session, username) {
          println!("Failed to reinstall Docker: {}", e);
          return Err(e);
        }
        println!("Docker successfully reinstalled!");
      }
    }
    Ok(())
  }

  fn _install_docker(&self, session: &Session, username: &str) -> anyhow::Result<()> {
    // Step 1: Remove potentially conflicting packages
    println!("Removing conflicting packages...");
    let remove_cmd = "sudo apt-get remove -y docker.io docker-doc docker-compose docker-compose-v2 podman-docker containerd runc";
    let (exit_code, output) = SSH::execute_command(session, remove_cmd)?;
    if exit_code != 0 {
      println!("Warning during package removal: {}", output);
      // Continue anyway as these might not be installed
    }

    // Step 2: Update package index and install prerequisites
    println!("Installing prerequisites...");
    SSH::execute_command(session, "sudo apt-get update")?;
    SSH::execute_command(session, "sudo apt-get install -y ca-certificates curl")?;

    // Step 3: Set up Docker's GPG key
    println!("Setting up Docker's GPG key...");
    SSH::execute_command(session, "sudo install -m 0755 -d /etc/apt/keyrings")?;
    SSH::execute_command(
      session,
      "sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc",
    )?;
    SSH::execute_command(session, "sudo chmod a+r /etc/apt/keyrings/docker.asc")?;

    // Step 4: Add Docker repository
    println!("Adding Docker repository...");
    let add_repo_cmd = r#"echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null"#;
    let (exit_code, output) = SSH::execute_command(session, add_repo_cmd)?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!(
        "Failed to add Docker repository: {}",
        output
      ));
    }

    // Step 5: Update package index again
    println!("Updating package index...");
    let (exit_code, output) = SSH::execute_command(session, "sudo apt-get update")?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!(
        "Failed to update package index: {}",
        output
      ));
    }

    // Step 6: Install Docker
    println!("Installing Docker packages...");
    let install_cmd = "sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin";
    let (exit_code, output) = SSH::execute_command(session, install_cmd)?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!("Failed to install Docker: {}", output));
    }

    // Step 7: Add user to docker group
    println!("Adding user to docker group...");
    SSH::execute_command(session, "sudo groupadd docker")?; // This might fail if group already exists, but that's okay
    let add_user_cmd = format!("sudo usermod -aG docker {}", username);
    let (exit_code, output) = SSH::execute_command(session, &add_user_cmd)?;
    if exit_code != 0 {
      return Err(anyhow::anyhow!(
        "Failed to add user to docker group: {}",
        output
      ));
    }

    // Step 8: Verify docker installation
    println!("Verifying Docker installation...");
    let (exit_code, docker_version) = SSH::execute_command(session, "docker --version")?;
    if exit_code != 0 || docker_version.is_empty() {
      return Err(anyhow::anyhow!("Docker installation verification failed"));
    }

    println!("Docker version: {}", docker_version.trim());
    Ok(())
  }

  pub fn save_to_cluster(&self, session: &Session) -> anyhow::Result<()> {
    let cluster_init_json = serde_json::to_string_pretty(self)
      .context("Failed to serialize ClusterInit to JSON")?
      .replace("'", "'\\''");
    let write_cmd = format!(
      "mkdir -p {} && cat > {} << 'EOF'\n{}\nEOF",
      REMOTE_CLUSTER_DAEMON_FOLDER, REMOTE_CLUSTER_INIT_FILE, cluster_init_json
    );
    SSH::execute_command(session, &write_cmd)?;
    Ok(())
  }
}
