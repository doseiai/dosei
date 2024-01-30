pub(crate) mod route;
pub(crate) mod schema;

use cached::{Cached, TimedCache};
use chrono::{DateTime, Utc};
use instant_acme::{
  Account, AccountCredentials, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, Order,
  OrderStatus,
};
use once_cell::sync::Lazy;
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use sqlx::testing::TestTermination;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;
use uuid::Uuid;

const CACHE_LIFESPAN: u64 = 600;
const INTERNAL_CHECK_SPAN: u64 = 5;
const EXTERNAL_MAX_CHECKS: u64 = 10;

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
  owner_id: Uuid,
  domain_name: &str,
  credentials: AccountCredentials,
  pool: Arc<Pool<Postgres>>,
) -> anyhow::Result<()> {
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
  let http1_challenge_token_cache = Arc::clone(&HTTP1_CHALLENGE_TOKEN_CACHE);
  {
    let mut cache = http1_challenge_token_cache.lock().await;
    cache.cache_set(
      challenge.token.clone(),
      order.key_authorization(challenge).as_str().to_string(),
    );
  }
  internal_check(
    owner_id,
    domain_name,
    &challenge.token,
    order.key_authorization(challenge).as_str(),
    Arc::new(Mutex::new(order)),
    pool,
  );
  Ok(())
}

pub fn internal_check(
  owner_id: Uuid,
  domain_name: &str,
  token: &str,
  token_value: &str,
  order: Arc<Mutex<Order>>,
  pool: Arc<Pool<Postgres>>,
) {
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
                external_check(owner_id, &domain_name, order, pool);
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

pub fn external_check(
  owner_id: Uuid,
  domain_name: &str,
  order: Arc<Mutex<Order>>,
  pool: Arc<Pool<Postgres>>,
) {
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
          drop(order_guard);
          match provision_certification(owner_id, &domain_name, order, pool).await {
            Ok(_) => {}
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

async fn provision_certification(
  owner_id: Uuid,
  domain_name: &str,
  order: Arc<Mutex<Order>>,
  pool: Arc<Pool<Postgres>>,
) -> anyhow::Result<()> {
  let certificate = {
    let mut params = CertificateParams::new(vec![domain_name.to_owned()]);
    params.distinguished_name = DistinguishedName::new();
    Certificate::from_params(params)?
  };
  let signing_request = certificate.serialize_request_der()?;
  let mut order = order.lock().await;
  order.finalize(&signing_request).await?;

  let cert_chain_pem = loop {
    match order.certificate().await? {
      Some(cert_chain_pem) => break cert_chain_pem,
      None => sleep(Duration::from_secs(1)).await,
    }
  };

  let mut certificates: Vec<String> = cert_chain_pem
    .split("-----END CERTIFICATE-----")
    .map(|cert| format!("{}-----END CERTIFICATE-----", cert))
    .collect();
  certificates.pop();

  let mut expires_at = Utc::now();
  if let Ok(cert) = openssl::x509::X509::from_pem(certificates[0].as_bytes()) {
    let not_after_str = cert.not_after().to_string().replace("GMT", "+0000");
    if let Ok(not_after) = DateTime::parse_from_str(&not_after_str, "%b %d %H:%M:%S %Y %z") {
      expires_at = not_after.with_timezone(&Utc);
    }
  }

  let certificate = schema::Certificate {
    id: Uuid::new_v4(),
    domain_name: domain_name.to_string(),
    certificate: certificates[0].to_string(),
    private_key: certificate.serialize_private_key_pem(),
    expires_at,
    owner_id,
    updated_at: Utc::now(),
    created_at: Utc::now(),
  };

  match sqlx::query_as!(
      schema::Certificate,
      r#"INSERT INTO certificate (id, domain_name, certificate, private_key, expires_at, owner_id, updated_at, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
       RETURNING id, domain_name, certificate, private_key, expires_at, owner_id, updated_at, created_at"#,
      certificate.id,
      certificate.domain_name,
      certificate.certificate,
      certificate.private_key,
      certificate.expires_at,
      certificate.owner_id,
      certificate.updated_at,
      certificate.created_at,
    ).fetch_one(&*pool).await {
    Ok(recs) => {
      info!("{:?}", recs);
    },
    Err(err) => {
      error!("Error in creating certificate: {:?}", err);
    }
  }
  // TODO: Send email and notify.
  Ok(())
}

async fn get_http01_challenge_token_value(token: String) -> Option<String> {
  let http1_challenge_token_cache = Arc::clone(&HTTP1_CHALLENGE_TOKEN_CACHE);
  {
    let mut cache = http1_challenge_token_cache.lock().await;
    if let Some(value) = cache.cache_get(&token) {
      let token_value = value.clone();
      return cache.cache_set(token, token_value);
    }
  }
  None
}

static HTTP1_CHALLENGE_TOKEN_CACHE: Lazy<Arc<Mutex<TimedCache<String, String>>>> =
  Lazy::new(|| {
    let cache = TimedCache::with_lifespan(CACHE_LIFESPAN);
    Arc::new(Mutex::new(cache))
  });
