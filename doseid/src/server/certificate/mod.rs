pub(crate) mod route;
pub(crate) mod schema;

use cached::{Cached, TimedCache};
use instant_acme::{
  Account, AccountCredentials, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, Order,
  OrderStatus,
};
use once_cell::sync::Lazy;
use rcgen::{Certificate, CertificateParams, DistinguishedName};
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
const EXTERNAL_MAX_CHECKS: u64 = 5;

pub fn external_check(domain_name: &str, order: Arc<Mutex<Order>>) {
  let domain_name = domain_name.to_string();
  let order = Arc::clone(&order);

  let mut attempts = 1;
  let mut backoff_duration = Duration::from_millis(250);
  tokio::spawn(async move {
    loop {
      sleep(backoff_duration).await;
      let mut order_guard = order.lock().await;
      let order_state = order_guard.refresh().await.unwrap();
      match order_state.status {
        OrderStatus::Ready => {
          info!("It's ready, begin create cert");
          drop(order_guard);
          match create_certification(&domain_name, order).await {
            Ok((certificate, private_key)) => {
              info!(certificate);
              info!(private_key);
            }
            Err(_) => {
              error!("Something went wrong when generating CERT");
            }
          };
          break;
        }
        order_status => {
          error!("Order Status: {:?}", order_status);
          backoff_duration *= 4;
          attempts += 1;

          if EXTERNAL_MAX_CHECKS <= attempts {
            error!("Order is not yet ready after {EXTERNAL_MAX_CHECKS} attempts, Giving up.");
            break;
          }
          info!("Order is not ready, waiting {backoff_duration:?}");
        }
      }
    }
  });
}

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
      if let Ok(response) = resolver.lookup_ip(&domain_name).await {
        if let Some(address) = response.iter().next() {
          let url = format!("http://{}/.well-known/acme-challenge/{}", address, token);
          let client = reqwest::Client::builder().no_proxy().build().unwrap();
          let response = client.get(&url).send().await;

          if response.is_success() {
            if let Ok(response_text) = response.unwrap().text().await {
              if response_text == token_value {
                external_check(&domain_name, order);
                break;
              }
            }
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

async fn create_certification(
  domain_name: &str,
  order: Arc<Mutex<Order>>,
) -> anyhow::Result<(String, String)> {
  info!("beging stuff");
  let certificate = {
    let mut params = CertificateParams::new(vec![domain_name.to_owned()]);
    params.distinguished_name = DistinguishedName::new();
    Certificate::from_params(params)?
  };

  info!("here");
  let signing_request = certificate.serialize_request_der()?;
  info!("here2");
  let mut order = order.lock().await;
  info!("here3");
  order.finalize(&signing_request).await?;

  info!("here4");
  let cert_chain_pem = loop {
    match order.certificate().await? {
      Some(cert_chain_pem) => break cert_chain_pem,
      None => sleep(Duration::from_secs(1)).await,
    }
  };
  info!(
    "Certificate and Private Key generated for domain: {}",
    domain_name
  );
  Ok((
    cert_chain_pem.to_string(),
    certificate.serialize_private_key_pem(),
  ))
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
