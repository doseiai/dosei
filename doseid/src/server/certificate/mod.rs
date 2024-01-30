pub(crate) mod route;
pub(crate) mod schema;

use cached::{Cached, TimedCache};
use instant_acme::{
  Account, AccountCredentials, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, Order,
  OrderStatus,
};
use once_cell::sync::Lazy;
use sqlx::testing::TestTermination;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

const CACHE_LIFESPAN: u64 = 600;
const INTERNAL_CHECK_SPAN: u64 = 5;

pub fn internal_check(domain_name: &str, token: &str, token_value: &str, order: Arc<Mutex<Order>>) {
  let domain_name = domain_name.to_string();
  let token = token.to_string();
  let token_value = token_value.to_string();
  let order = Arc::clone(&order);

  let mut attempts = 0;
  tokio::spawn(async move {
    loop {
      sleep(Duration::from_secs(INTERNAL_CHECK_SPAN)).await;
      let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
      let response = resolver.lookup_ip(&domain_name).await.unwrap();
      let address = response.iter().next().expect("no addresses returned!");
      let url = format!("http://{}/.well-known/acme-challenge/{}", address, token);
      let client = reqwest::Client::builder().no_proxy().build().unwrap();
      let response = client.get(&url).send().await;

      if response.is_success() {
        if let Ok(response_text) = response.unwrap().text().await {
          if response_text == token_value {
            let mut order = order.lock().await;
            let order_state = order.refresh().await.unwrap();
            match order_state.status {
              OrderStatus::Ready => {
                info!("Order Status Ready, TODO, genete cert");
              }
              _ => {
                error!("Give up, It's you not me");
              }
            }
            break;
          }
        }
      }
      error!("Failed to get fetch {}, trying again", &domain_name);
      if CACHE_LIFESPAN <= attempts {
        error!("Too many tries, giving up");
        break;
      }
      attempts += INTERNAL_CHECK_SPAN;
    }
  });
}

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
  internal_check(
    domain_name,
    &challenge.token,
    order.key_authorization(challenge).as_str(),
    Arc::new(Mutex::new(order)),
  );
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
  let cache = TimedCache::with_lifespan(CACHE_LIFESPAN);
  Arc::new(Mutex::new(cache))
});
