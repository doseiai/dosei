use crate::cluster;
use crate::config::{ClusterConfig, Config};
use crate::init::InitTemplate;
use anyhow::anyhow;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::io;
use std::io::Write;

#[derive(Parser)]
#[command(
  name= "dosei",
  version = env!("CARGO_PKG_VERSION"),
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Initialize a new project from a template in an existing directory
  Init {
    /// The target directory path
    #[arg(index = 1, default_value = ".")]
    path: String,

    /// The template name (if not provided, will prompt for selection)
    #[arg(short = 't', long = "template", value_enum)]
    template: Option<InitTemplate>,
  },
  // /// Run a Dosei App
  // Run {
  //   /// Expose while running
  //   #[arg(long = "expose")]
  //   expose: bool,
  // },
  /// Deploy a Dosei App
  Deploy {
    /// Cluster name
    cluster_name: Option<String>,
    /// Deploy even if the working directory is dirty
    #[arg(long = "allow-dirty")]
    allow_dirty: bool,
  },
  /// Cluster commands
  Cluster {
    #[clap(subcommand)]
    command: cluster::command::Commands,
  },
  // /// Environment variables commands
  // Env {
  //   #[clap(subcommand)]
  //   command: env::Commands,
  // },
  /// Output shell completion script to standard output.
  Completion {
    #[arg(value_enum, index = 1)]
    shell: Shell,
  },
}

impl Cli {
  pub fn get_default_cluster_or_ask(
    name: Option<String>,
  ) -> anyhow::Result<(String, ClusterConfig)> {
    let config = Config::load()?;

    let cluster_name = if let Some(name) = name {
      name
    } else if let Ok(env_cluster) = std::env::var("DOSEI_CLUSTER") {
      env_cluster
    } else if let Some(default_cluster_map) = config.get_default_cluster() {
      default_cluster_map.keys().next().unwrap().clone()
    } else {
      // No default found, prompt user
      let mut input = String::new();
      print!("Enter the cluster name: ");
      io::stdout().flush()?;
      io::stdin().read_line(&mut input)?;
      input.trim().to_string()
    };

    // If DOSEI_SSH_KEY is set (CI/CD), allow using a cluster not in the config
    if let Some(cluster) = config.get_cluster(&cluster_name) {
      Ok((cluster_name, cluster.clone()))
    } else if std::env::var("DOSEI_SSH_KEY").is_ok() {
      // In CI/CD mode: cluster isn't in local config but we have the key via env
      Ok((
        cluster_name,
        ClusterConfig {
          id: None,
          username: String::from("dosei"),
          ssh_key: None, // Key will come from DOSEI_SSH_KEY env var
        },
      ))
    } else {
      Err(anyhow!("Cluster {} not found", cluster_name))
    }
  }
  pub fn check_allow_dirty(allow_dirty: bool) -> anyhow::Result<()> {
    // Check for uncommitted changes
    if !allow_dirty {
      // Open the git repository in the current directory
      let repo = git2::Repository::open_from_env()?;

      // Get the repository status
      let mut status_options = git2::StatusOptions::new();
      status_options
        .include_untracked(true)
        .recurse_untracked_dirs(true);

      let statuses = repo.statuses(Some(&mut status_options))?;

      if !statuses.is_empty() {
        // Get the list of dirty files
        let dirty_files: Vec<String> = statuses
          .iter()
          .filter_map(|entry| entry.path().map(|path| format!("  * {} (dirty)", path)))
          .collect();

        // Format the error message
        let files_list = dirty_files.join("\n");
        let mut cmd = Cli::command();
        cmd.error(
          ErrorKind::ValueValidation,
          format!("the working directory has uncommitted changes; if you'd like to suppress this error pass `--allow-dirty`, or commit the changes to these files:\n\n{}", files_list)
        ).exit();
      }
    }
    Ok(())
  }
}
