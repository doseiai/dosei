pub(crate) mod schema;

use ring::aead::{Aad, LessSafeKey, Nonce};
use ring::error::Unspecified;
use uuid::Uuid;

pub fn encrypt_value(
  owner_id: Uuid,
  value: &str,
  sealing_key: &mut LessSafeKey,
  nonce: Nonce,
) -> Result<Vec<u8>, Unspecified> {
  let mut in_out = value.as_bytes().to_vec();
  sealing_key.seal_in_place_append_tag(nonce, Aad::from(owner_id.as_bytes()), &mut in_out)?;
  Ok(in_out)
}

pub fn decrypt_value(
  owner_id: Uuid,
  encrypted_value: &[u8],
  opening_key: &mut LessSafeKey,
  nonce: Nonce,
) -> Result<String, Unspecified> {
  let associated_data = Aad::from(Aad::from(owner_id.as_bytes()));
  let mut in_out = encrypted_value.to_vec();
  let decrypted_data = opening_key.open_in_place(nonce, associated_data, &mut in_out)?;
  String::from_utf8(decrypted_data.to_vec()).map_err(|_| Unspecified)
}
