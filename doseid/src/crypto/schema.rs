use ring::aead::{LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};

pub struct SigningKey {
  pub key: LessSafeKey,
  pub bytes: Vec<u8>,
  pub nonce: Nonce,
}

impl SigningKey {
  pub fn new() -> anyhow::Result<SigningKey> {
    let mut key_bytes = vec![0; AES_256_GCM.key_len()];
    SystemRandom::new().fill(&mut key_bytes)?;
    let unbounded_key = UnboundKey::new(&AES_256_GCM, &key_bytes)?;

    let mut nonce_bytes = vec![0; NONCE_LEN];
    SystemRandom::new().fill(&mut nonce_bytes)?;
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)?;

    Ok(SigningKey {
      key: LessSafeKey::new(unbounded_key),
      bytes: key_bytes,
      nonce,
    })
  }
  pub fn fill(key: Vec<u8>, nonce_vector: Vec<u8>) -> anyhow::Result<SigningKey> {
    let unbounded_key = UnboundKey::new(&AES_256_GCM, &key)?;
    let nonce = Nonce::try_assume_unique_for_key(&nonce_vector)?;

    Ok(SigningKey {
      key: LessSafeKey::new(unbounded_key),
      bytes: key,
      nonce,
    })
  }
}
