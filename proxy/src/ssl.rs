//! ACME client implementation for Dosei using Let's Encrypt
//! Provisions new certs and attach them to projects
//! Uses for http01 and DNS based challenges
//! Tries to renew the certificate if above a set age
//! Tries to keep the Let's Encrypt API limits in check
//!

use anyhow::Ok;
use std::time::Duration;
use tokio::time::sleep;

use instant_acme::{
  Account, AccountCredentials, Authorization, AuthorizationStatus, Challenge, ChallengeType,
  Identifier, LetsEncrypt, NewAccount, NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use tracing::{error, info};

const MAX_RETRIES: usize = 20;
const MAX_CERTIFICATE_RETRIES: usize = 5;

// create account and get creds
pub async fn create_account(email: &str) -> Result<AccountCredentials, anyhow::Error> {
  let server_url = LetsEncrypt::Staging.url().to_string();

  let new_account = &NewAccount {
    contact: &[&format!("mailto:{email}")],
    terms_of_service_agreed: true,
    only_return_existing: false,
  };

  let (_account, credentials) = Account::create(new_account, &server_url, None)
    .await
    .unwrap();

  Ok(credentials)
}

// create certificate and get it's private key
// for wildcard domains, DNS challenges are used
// for simple domains, HTTP challenge holds fine.
pub async fn create_certificate(
  identifier: &str,
  // challenge_type: ChallengeType,
  // account: Account,
  credentials: AccountCredentials,
) -> Result<String, anyhow::Error> {
  // place an order
  let mut certificate_order = Account::from_credentials(credentials)
    .await?
    .new_order(&NewOrder {
      identifiers: &[Identifier::Dns(identifier.to_string())],
    })
    .await?;

  let state = certificate_order.state();
  info!("order state: {:#?}", state);
  assert!(matches!(state.status, OrderStatus::Pending));

  // add order authorization
  let authorizations = certificate_order.authorizations().await?;

  let authorization = authorizations.first().unwrap();
  match authorization.status {
    AuthorizationStatus::Pending => {
      info!("auth status pending")
    }
    AuthorizationStatus::Valid => {
      info!("auth status valid, proceeding.")
    }
    _ => todo!(),
  }

  // complete acme challenge
  // complete_acme_challenge(challenge_type, authorization, &mut certificate_order).await?;

  // let server know system is ready to accept challenges
  let dns_challenge = authorization
    .challenges
    .iter()
    .find(|ch| ch.r#type == ChallengeType::Dns01)
    .unwrap();

  println!(
    "_acme-challenge.{} IN TXT {}",
    identifier,
    certificate_order
      .key_authorization(dns_challenge)
      .dns_value()
  );

  certificate_order
    .set_challenge_ready(&dns_challenge.url)
    .await?;

  // exponential backoff
  await_order_completion(&mut certificate_order).await?;

  let mut cert_params = CertificateParams::new(vec![identifier.to_owned()]);
  cert_params.distinguished_name = DistinguishedName::new();

  let certificate = Certificate::from_params(cert_params)?;
  let certificate_signing_request = certificate.serialize_request_der()?;

  certificate_order
    .finalize(&certificate_signing_request)
    .await?;

  // certifcate polling
  let mut res: Option<String> = None;
  let mut retries = MAX_CERTIFICATE_RETRIES;
  while res.is_none() && retries > 0 {
    res = certificate_order.certificate().await?;
    retries -= 1;
    sleep(Duration::from_secs(1)).await;
  }

  Ok(certificate.serialize_private_key_pem())
}

pub async fn await_order_completion(order: &mut Order) -> anyhow::Result<()> {
  let mut tries = 1;
  let mut delay = Duration::from_millis(250);
  loop {
    sleep(delay).await;

    let state = order.refresh().await?;

    if let OrderStatus::Ready | OrderStatus::Invalid = state.status {
      info!("order state: {:#?}", state);
      break;
    }

    delay *= 2;
    tries += 1;
    match tries < MAX_RETRIES {
      true => info!("order is not ready, waiting {delay:?}"),
      false => {
        error!("order is not yet ready in {MAX_RETRIES} tries");
        return Err(anyhow::anyhow!("order is not ready"));
      }
    }
  }

  let order_state = order.state();
  if order_state.status != OrderStatus::Ready {
    error!("unexpected order status: {:?}", order_state.status)
  }
  Ok(())
}
