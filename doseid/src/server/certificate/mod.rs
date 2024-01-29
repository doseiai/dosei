pub(crate) mod route;
pub(crate) mod schema;

use cached::{Cached, TimedCache};
use instant_acme::{
  Account, AccountCredentials, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder,
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub async fn create_acme_account(email: &str) -> anyhow::Result<AccountCredentials> {
  let server_url = LetsEncrypt::Staging.url().to_string();

  let new_account_info = NewAccount {
    contact: &[&format!("mailto:{}", email)],
    terms_of_service_agreed: true,
    only_return_existing: false,
  };

  let result = Account::create(&new_account_info, &server_url, None).await?;
  Ok(result.1)
}

pub async fn create_acme_certificate(
  domain_name: &str,
  credentials: AccountCredentials,
) -> anyhow::Result<String> {
  let mut order = Account::from_credentials(credentials)
    .await?
    .new_order(&NewOrder {
      identifiers: &[Identifier::Dns(domain_name.to_string())],
    })
    .await?;

  let authorizations = order.authorizations().await?;
  let authorization = &authorizations
    .first()
    .ok_or_else(|| anyhow::Error::msg("authorization not found"))?;
  let challenge = authorization
    .challenges
    .iter()
    .find(|ch| ch.r#type == ChallengeType::Http01)
    .ok_or_else(|| anyhow::Error::msg("http-01 challenge not found"))?;
  order.set_challenge_ready(&challenge.url).await?;
  let certificate_order_cache = Arc::clone(&CERTIFICATE_ORDER_CACHE);
  {
    let mut cache = certificate_order_cache.lock().await;
    cache.cache_set(
      challenge.token.clone(),
      order.key_authorization(challenge).as_str().to_string(),
    );
  }
  Ok(challenge.token.clone())
}

async fn get_certificate_order(token: String) -> Option<String> {
  let certificate_order_cache = Arc::clone(&CERTIFICATE_ORDER_CACHE);
  {
    let mut cache = certificate_order_cache.lock().await;
    if let Some(value) = cache.cache_get(&token) {
      let token_value = value.clone();
      return cache.cache_set(token, token_value);
    }
  }
  None
}

static CERTIFICATE_ORDER_CACHE: Lazy<Arc<Mutex<TimedCache<String, String>>>> = Lazy::new(|| {
  let cache = TimedCache::with_lifespan(600);
  Arc::new(Mutex::new(cache))
});
