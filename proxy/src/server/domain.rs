use sqlx::{Error, Pool, Postgres};
use std::sync::Arc;

pub async fn get_domain(host: String, pool: Arc<Pool<Postgres>>) -> Option<i16> {
  // Reserved ones
  let wildcard_domain = ".dosei.app";
  if host.ends_with(&wildcard_domain) {
    let host_split: Vec<&str> = host.split(wildcard_domain).collect();
    if let Some(subdomain) = host_split.first() {
      let subdomain_split: Vec<&str> = subdomain.split('-').collect();
      if subdomain_split.len() >= 2 {
        let account_name = subdomain_split.first().unwrap();
        let service_name = subdomain.replace(&format!("{}-", account_name), "");
        match sqlx::query!(
          "
        SELECT
        account.name AS account_name,
        service.name AS service_name,
        deployment.host_port AS service_port
        FROM account
        JOIN service ON account.id = service.owner_id
        JOIN deployment ON account.id = deployment.owner_id
        WHERE account.name = $1 AND service.name = $2 AND deployment.status = 'running'
        ",
          account_name,
          service_name
        )
        .fetch_one(&*pool)
        .await
        {
          Ok(record) => {
            return record.service_port;
          }
          Err(error) => match &error {
            Error::RowNotFound => {}
            _ => {
              return None;
            }
          },
        };
      }
    }
  }
  None
}
