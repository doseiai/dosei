//! ACME client implementation for Dosei using Let's Encrypt
//! Provisions new certs and attach them to projects
//! Uses for http01 and DNS based challenges
//! Tries to renew the certificate if above a set age
//! Tries to keep the Let's Encrypt API limits in check
//!

use anyhow::Ok;
use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::time::sleep;

use instant_acme::{
  Account, AccountCredentials, Authorization, AuthorizationStatus, Challenge, ChallengeType,
  Identifier, LetsEncrypt, NewAccount, NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use tracing::{error, info};

const MAX_WAIT_ATTEMPTS: usize = 20;
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

  wait_for_completed_order(&mut certificate_order).await?;

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

pub async fn wait_for_completed_order(order: &mut Order) -> Result<()> {
  let mut attempts = 1;
  let mut backoff_duration = Duration::from_millis(250);

  loop {
    sleep(backoff_duration).await;

    let order_state = order.refresh().await?;

    match order_state.status {
      OrderStatus::Ready | OrderStatus::Invalid => {
        info!("Order state: {:#?}", order_state);
        break;
      }
      _ => {
        backoff_duration *= 2;
        attempts += 1;

        if attempts < MAX_WAIT_ATTEMPTS {
          info!("Order is not ready, waiting {backoff_duration:?}");
        } else {
          error!("Order is not yet ready after {MAX_WAIT_ATTEMPTS} attempts");
          return Err(anyhow!("Order is not ready"));
        }
      }
    }
  }

  let final_order_state = order.state();
  if final_order_state.status != OrderStatus::Ready {
    error!("Unexpected order status: {:?}", final_order_state.status);
  }

  Ok(())
}
