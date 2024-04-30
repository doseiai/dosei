use crate::crypto::schema::SigningKey;
use crate::crypto::{decrypt_value, encrypt_value};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Env {
  pub id: Uuid,
  pub name: String,
  pub value: String,
  pub key: Option<String>,
  pub nonce: Option<String>,
  pub service_id: Option<Uuid>,
  pub deployment_id: Option<Uuid>,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Env {
  async fn save_env(&self, pool: Arc<Pool<Postgres>>) -> anyhow::Result<()> {
    sqlx::query_as!(
      Env,
      "
      INSERT INTO env (
      id, name, value, key, nonce, deployment_id, service_id, owner_id, updated_at, created_at
      )
      VALUES (
      $1::uuid, $2, $3, $4, $5, $6::uuid, $7::uuid, $8::uuid, $9, $10
      )
      RETURNING *
      ",
      self.id,
      self.name,
      self.value,
      self.key,
      self.nonce,
      self.deployment_id,
      self.service_id,
      self.owner_id,
      self.updated_at,
      self.created_at
    )
    .fetch_one(&*pool)
    .await?;
    Ok(())
  }

  async fn get_env(
    name: String,
    value: String,
    owner_id: Uuid,
    pool: Arc<Pool<Postgres>>,
  ) -> anyhow::Result<Env> {
    let env = sqlx::query_as!(
      Env,
      "SELECT * FROM env WHERE name = $1 and value = $2 and owner_id = $3::uuid",
      name,
      value,
      owner_id
    )
    .fetch_one(&*pool)
    .await?;
    Ok(env)
  }

  pub async fn save_secret_encrypted(&mut self, pool: Arc<Pool<Postgres>>) -> anyhow::Result<()> {
    let mut signing_key = SigningKey::new()?;

    let nonce_base64 = general_purpose::STANDARD.encode(*signing_key.nonce.as_ref());
    let encrypted_value: Vec<u8> = encrypt_value(
      self.owner_id,
      &self.value,
      &mut signing_key.key,
      signing_key.nonce,
    )?;
    self.value = general_purpose::STANDARD.encode(encrypted_value);
    self.key = Some(general_purpose::STANDARD.encode(signing_key.bytes));
    self.nonce = Some(nonce_base64);

    self.save_env(pool).await?;
    Ok(())
  }

  pub async fn get_secret_decrypted(
    name: String,
    value: String,
    owner_id: Uuid,
    pool: Arc<Pool<Postgres>>,
  ) -> anyhow::Result<Env> {
    let mut secret = Env::get_env(name, value, owner_id, pool).await?;

    let decoded_key: Vec<u8> =
      general_purpose::STANDARD.decode(secret.key.clone().unwrap().as_bytes())?;
    let decoded_value: Vec<u8> = general_purpose::STANDARD.decode(secret.value.clone())?;
    let decoded_nonce: Vec<u8> =
      general_purpose::STANDARD.decode(secret.nonce.clone().unwrap().as_bytes())?;

    let mut opening_key = SigningKey::fill(decoded_key, decoded_nonce)?;

    secret.value = decrypt_value(
      secret.owner_id,
      &decoded_value,
      &mut opening_key.key,
      opening_key.nonce,
    )?;

    Ok(secret)
  }
}
