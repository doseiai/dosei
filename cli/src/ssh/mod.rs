use crate::file::expand_tilde;
use anyhow::{anyhow, Context};
use dosei_schema::ssh::{SSHBearerPayload, DOSEI_SSH_NAMESPACE};
use serde::{Deserialize, Serialize};
use ssh2::Session;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey};
use std::fs;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[allow(clippy::upper_case_acronyms)]
pub struct SSH;

#[derive(Debug, Serialize, Deserialize)]
pub struct CliSSHBearerPayload(pub SSHBearerPayload);

impl Deref for CliSSHBearerPayload {
  type Target = SSHBearerPayload;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl CliSSHBearerPayload {
  pub fn new(ssh_key_path: Option<PathBuf>) -> anyhow::Result<Self> {
    let private_key_data = if let Ok(key_content) = std::env::var("DOSEI_SSH_KEY") {
      // CI/CD: use inline key from environment variable
      key_content
    } else {
      match ssh_key_path {
        Some(path) => fs::read_to_string(path)?,
        None => fs::read_to_string(SSH::get_default_ssh_key_path()?)?,
      }
    };
    let private_key = PrivateKey::from_openssh(&private_key_data)?;
    let key_fingerprint = private_key
      .public_key()
      .fingerprint(HashAlg::default())
      .to_string();

    let nonce = Uuid::new_v4().to_string();
    let signature = private_key
      .sign(DOSEI_SSH_NAMESPACE, HashAlg::default(), nonce.as_bytes())
      .context("Failed to sign payload")?;

    Ok(Self(SSHBearerPayload {
      namespace: DOSEI_SSH_NAMESPACE.to_string(),
      nonce,
      key_fingerprint,
      signature: signature.signature_bytes().to_vec(),
    }))
  }
}

impl SSH {
  pub fn execute_command(session: &Session, command: &str) -> anyhow::Result<(i32, String)> {
    let mut channel = session
      .channel_session()
      .context("Failed to create channel session")?;

    channel
      .exec(command)
      .context(format!("Failed to execute command: {}", command))?;

    let mut output = String::new();
    channel
      .read_to_string(&mut output)
      .context("Failed to read command output")?;

    channel.wait_close().context("Failed to close channel")?;

    let exit_status = channel.exit_status().context("Failed to get exit status")?;

    Ok((exit_status, output))
  }

  pub fn check_file_exists(session: &Session, path: &str) -> anyhow::Result<bool> {
    let (exit_code, _) =
      Self::execute_command(session, &format!("[ -f {} ] && echo 'exists'", path))?;
    Ok(exit_code == 0)
  }

  pub fn get_default_ssh_key_path() -> anyhow::Result<PathBuf> {
    let ed25519_path = expand_tilde("~/.ssh/id_ed25519");
    if fs::read_to_string(&ed25519_path).is_ok() {
      return Ok(ed25519_path);
    }

    let rsa_path = expand_tilde("~/.ssh/id_rsa");
    if fs::read_to_string(&rsa_path).is_ok() {
      return Ok(rsa_path);
    }
    Err(anyhow!(
      "Could not find SSH keys. Tried ~/.ssh/id_ed25519 and ~/.ssh/id_rsa"
    ))
  }
  pub fn get_public_key_from_private_key_path(
    private_key_path: &Path,
  ) -> anyhow::Result<(String, String)> {
    let private_key_data = fs::read_to_string(private_key_path)?;
    Self::get_public_key_from_private_key(private_key_data)
  }
  pub fn get_public_key_from_private_key(
    pem: impl AsRef<[u8]>,
  ) -> anyhow::Result<(String, String)> {
    let private_key = PrivateKey::from_openssh(pem)?;
    let public_key = private_key.public_key();
    Ok((public_key.to_openssh()?, public_key.comment().to_string()))
  }

  pub fn generate_ed25519_key() -> anyhow::Result<String> {
    let mut rng = OsRng;
    let current_dir = std::env::current_dir()?;
    let dosei_dir = current_dir.join("../../../.dosei");
    if !dosei_dir.exists() {
      fs::create_dir_all(&dosei_dir).context("Failed to create .dosei directory")?;
    }

    let private_key_path = dosei_dir.join("dosei_ed25519");
    let public_key_path = dosei_dir.join("dosei_ed25519.pub");

    if private_key_path.exists() && public_key_path.exists() {
      return Ok(fs::read_to_string(public_key_path)?.trim().to_string());
    }

    let keypair = PrivateKey::random(&mut rng, Algorithm::Ed25519)?;
    let private_key = keypair.to_openssh(LineEnding::default())?;
    let public_key = keypair.public_key().to_openssh()?;

    fs::write(&private_key_path, private_key.as_bytes()).context(format!(
      "Failed to write key to {}",
      private_key_path.display()
    ))?;

    fs::write(&public_key_path, public_key.as_bytes()).context(format!(
      "Failed to write key to {}",
      public_key_path.display()
    ))?;

    // Set appropriate permissions (read/write for owner only)
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let metadata = fs::metadata(&private_key_path)?;
      let mut permissions = metadata.permissions();
      permissions.set_mode(0o600); // Read/write for owner only
      fs::set_permissions(&private_key_path, permissions)?;
    }

    Ok(public_key)
  }
}

#[cfg(test)]
mod tests {
  use crate::ssh::{CliSSHBearerPayload, SSH};
  use ssh_key::PrivateKey;
  use std::fs;

  #[test]
  fn verify_ssh_key_verify() {
    let private_key_data = fs::read_to_string(SSH::get_default_ssh_key_path().unwrap()).unwrap();
    let private_key = PrivateKey::from_openssh(&private_key_data).unwrap();

    let payload = CliSSHBearerPayload::new(None).unwrap();

    assert!(payload.verify(private_key.public_key()));
  }

  #[test]
  fn generate_dosei_key() {
    SSH::generate_ed25519_key().unwrap();
  }
}
