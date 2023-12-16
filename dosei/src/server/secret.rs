use crate::schema::Secret;
use axum::extract::Query;
use axum::{Extension, Json};
use ring::aead::{
  Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM, NONCE_LEN,
};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn api_get_envs(pool: Extension<Arc<Pool<Postgres>>>) -> Json<Vec<Secret>> {
  let recs = sqlx::query_as!(Secret, "SELECT * from envs")
    .fetch_all(&**pool)
    .await
    .unwrap();
  Json(recs)
}

pub async fn api_set_envs(
  pool: Extension<Arc<Pool<Postgres>>>,
  Query(query): Query<SetEnvsQueryParams>,
  Json(body): Json<HashMap<String, String>>,
) -> Json<HashMap<String, String>> {

  let rand = SystemRandom::new();

  // Generate a new symmetric encryption key
  let mut key_bytes = vec![0; AES_256_GCM.key_len()];
  rand.fill(&mut key_bytes).unwrap();
  let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes).unwrap();
  let nonce_sequence = CounterNonceSequence(1);
  let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

  let mut encrypted_envs = HashMap::new();

  for (key, value) in body.iter() {
    let encrypted_value = encrypt_value(value, &mut sealing_key).unwrap(); // Replace with proper error handling
    let encrypted_value_hex = hex::encode(&encrypted_value);

    // Insert encrypted value into the database
    // sqlx::query!("INSERT INTO envs (key, value) VALUES ($1, $2)", key, &encrypted_value_hex).execute(&**pool).await.unwrap(); // Replace with proper error handling

    encrypted_envs.insert(key.clone(), encrypted_value_hex);
  }

  println!("{:?}", body);
  println!("{query:?}");
  let rec = HashMap::new();
  Json(rec)
}

#[derive(Deserialize, Debug)]
pub struct SetEnvsQueryParams {
  owner_name: Option<String>,
  project_name: Option<String>,
}

fn get_key() -> anyhow::Result<String> {
  let rand = SystemRandom::new();
  let mut key_bytes = vec![0; AES_256_GCM.key_len()];
  rand.fill(&mut key_bytes)?;

  let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)?;

  // Create a new NonceSequence type which generates nonces
  let nonce_sequence = CounterNonceSequence(1);

  // Create a new AEAD key for encrypting and signing ("sealing"), bound to a nonce sequence
  // The SealingKey can be used multiple times, each time a new nonce will be used
  let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

  Ok()
}

fn encrypt_value(
  uuid: Uuid,
  value: &str,
  sealing_key: &mut SealingKey<CounterNonceSequence>
) -> Result<Vec<u8>, Unspecified> {
  let mut in_out = value.as_bytes().to_vec();
  sealing_key.seal_in_place_append_tag(Aad::from(uuid.as_bytes()), &mut in_out)?;
  Ok(in_out)
}

fn decrypt_value(
  uuid: Uuid,
  encrypted_value: &[u8],
  opening_key: &mut OpeningKey<CounterNonceSequence>
) -> Result<String, Unspecified> {
  let associated_data = Aad::from(Aad::from(uuid.as_bytes()));
  let mut in_out = encrypted_value.to_vec();
  let decrypted_data = opening_key.open_in_place(associated_data, &mut in_out)?;
  String::from_utf8(decrypted_data.to_vec()).map_err(|_| Unspecified)
}


struct CounterNonceSequence(u32);

impl NonceSequence for CounterNonceSequence {
  // called once for each seal operation
  fn advance(&mut self) -> Result<Nonce, Unspecified> {
    let mut nonce_bytes = vec![0; NONCE_LEN];

    let bytes = self.0.to_be_bytes();
    nonce_bytes[8..].copy_from_slice(&bytes);
    println!("nonce_bytes = {}", hex::encode(&nonce_bytes));

    self.0 += 1; // advance the counter
    Nonce::try_assume_unique_for_key(&nonce_bytes)
  }
}

pub fn encrypt_secret() -> Result<(), Unspecified> {
  // Create a new instance of SystemRandom to be used as the single source of entropy
  let rand = SystemRandom::new();

  // Generate a new symmetric encryption key
  let mut key_bytes = vec![0; AES_256_GCM.key_len()];
  rand.fill(&mut key_bytes)?;
  println!("key_bytes = {}", hex::encode(&key_bytes)); // don't print this in production code

  // Create a new AEAD key without a designated role or nonce sequence
  let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)?;

  // Create a new NonceSequence type which generates nonces
  let nonce_sequence = CounterNonceSequence(1);
  let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

  // This data will be authenticated but not encrypted
  //let associated_data = Aad::empty(); // is optional so can be empty
  let associated_data = Aad::from(b"additional public data");

  // Data to be encrypted
  let data = b"hello world";
  println!("data = {}", String::from_utf8(data.to_vec()).unwrap());

  // Create a mutable copy of the data that will be encrypted in place
  let mut in_out = *data;

  // Encrypt the data with AEAD using the AES_256_GCM algorithm
  let tag = sealing_key.seal_in_place_separate_tag(associated_data, &mut in_out)?;
  println!("encrypted_data = {:?} {:?}", in_out, hex::encode(in_out)); // Print the encrypted data

  // Recreate the previously moved variables
  let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)?;
  let nonce_sequence = CounterNonceSequence(1);
  //let associated_data = Aad::empty(); // supplying the wrong data causes the decryption to fail
  let associated_data = Aad::from(b"additional public data");

  // Create a new AEAD key for decrypting and verifying the authentication tag
  let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

  // Decrypt the data by passing in the associated data and the cypher text with the authentication tag appended
  let mut cypher_text_with_tag = [&in_out, tag.as_ref()].concat();
  let decrypted_data = opening_key.open_in_place(associated_data, &mut cypher_text_with_tag)?;
  println!(
    "decrypted_data = {}",
    String::from_utf8(decrypted_data.to_vec()).unwrap()
  );

  assert_eq!(data, decrypted_data);
  Ok(())
}
