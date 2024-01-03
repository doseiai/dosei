//! ACME client implementation for Dosei using Let's Encrypt
//! Provisions new certs and attach them to projects
//! Uses for http01 and DNS based challenges
//! Tries to renew the certificate if above a set age
//! Tries to keep the Let's Encrypt API limits in check
//!

use anyhow::{anyhow, Result};
use std::error::Error;
use std::fmt;
use std::time::Duration;
use tokio::time::sleep;

use instant_acme::{
  Account, AccountCredentials, Authorization, AuthorizationStatus, ChallengeType, Identifier,
  LetsEncrypt, NewAccount, NewOrder, Order, OrderStatus,
};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use tracing::{error, info};

const MAX_WAIT_ATTEMPTS: usize = 10;

// Custom error type for better error reporting
#[derive(Debug)]
struct AccountCreationError;

impl fmt::Display for AccountCreationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Failed to create a new account.")
  }
}

impl Error for AccountCreationError {}

// Create an account and retrieve credentials
pub async fn create_account(email: &str) -> Result<AccountCredentials, anyhow::Error> {
  // Use the staging server URL from LetsEncrypt
  let staging_server_url = LetsEncrypt::Staging.url().to_string();

  // Define the new account information
  let new_account_info = NewAccount {
    contact: &[&format!("mailto:{}", email)],
    terms_of_service_agreed: true,
    only_return_existing: false,
  };

  // Create a new account and obtain credentials
  let result = Account::create(&new_account_info, &staging_server_url, None).await;

  match result {
    Ok((_, credentials)) => Ok(credentials),
    Err(_) => Err(AccountCreationError.into()), // Convert the error to a custom error type
  }
}

// create certificate and get it's private key
pub async fn create_certificate(
  identifier: &str,
  credentials: AccountCredentials,
) -> Result<String, anyhow::Error> {
  // place an order
  let mut order = Account::from_credentials(credentials)
    .await?
    .new_order(&NewOrder {
      identifiers: &[Identifier::Dns(identifier.to_string())],
    })
    .await?;

  let state = order.state();
  info!("order state: {:#?}", state);
  assert!(matches!(state.status, OrderStatus::Pending));

  // add order authorization
  let authorizations = order.authorizations().await?;

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

  // complete acme dns01 challenge
  perform_dns_challenge(authorization, identifier, &mut order).await?;

  let mut cert_params = CertificateParams::new(vec![identifier.to_owned()]);
  cert_params.distinguished_name = DistinguishedName::new();

  let certificate = Certificate::from_params(cert_params)?;
  let certificate_signing_request = certificate.serialize_request_der()?;

  order.finalize(&certificate_signing_request).await?;

  // certifcate polling
  let _cert_chain_pem = loop {
    match order.certificate().await.unwrap() {
      Some(cert_chain_pem) => break cert_chain_pem,
      None => sleep(Duration::from_secs(1)).await,
    }
  };

  info!("certificate: {}", certificate.serialize_private_key_pem());
  Ok(certificate.serialize_private_key_pem())
}

async fn wait_for_completed_order(order: &mut Order) -> Result<()> {
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
        backoff_duration *= 4;
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

// complete the dns challenge
async fn perform_dns_challenge(
  authorization: &Authorization,
  identifier: &str,
  order: &mut Order,
) -> anyhow::Result<()> {
  let dns_challenge = authorization
    .challenges
    .iter()
    .find(|ch| ch.r#type == ChallengeType::Dns01)
    .unwrap();

  info!(
    "_acme-challenge.{} IN TXT {}",
    identifier,
    order.key_authorization(dns_challenge).dns_value()
  );
  info!("delaying for 90 secs to allow user to follow instructions");
  sleep(Duration::from_secs(90)).await;

  order.set_challenge_ready(&dns_challenge.url).await?;

  wait_for_completed_order(order).await
}
