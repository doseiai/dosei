mod dashboard;

use crate::account::{Account, AccountSSHKey};
use crate::cluster::dashboard::Dashboard;
use crate::deployment::Deployment;
use crate::ingress::Ingress;
use crate::service::Service;
use dosei_schema::cluster::ClusterInit;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use utoipa::gen::serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonClusterInit(pub ClusterInit);

impl Deref for DaemonClusterInit {
  type Target = ClusterInit;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Clone)]
pub struct Cluster {
  pub name: String,
}

pub static CLUSTER: Lazy<Arc<Mutex<Cluster>>> = Lazy::new(|| {
  Arc::new(Mutex::new(Cluster {
    name: "localhost".to_string(),
  }))
});

impl Cluster {
  pub async fn init(name: String) {
    let mut cluster = CLUSTER.lock().await;
    cluster.name = name;
  }
  pub async fn get() -> Cluster {
    CLUSTER.lock().await.clone()
  }
}

impl DaemonClusterInit {
  pub async fn new() -> anyhow::Result<Self> {
    let cluster_init_path = Path::new("/var/lib/doseid/cluster-init.json");
    let cluster_data = fs::read_to_string(&cluster_init_path).await?;
    Ok(serde_json::from_str::<Self>(&cluster_data)?)
  }
  pub async fn init(&self, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    Cluster::init(self.name.clone()).await;
    let _ = Account::new("dosei", None, pg_pool).await;
    let default_user = Account::get_default_user(pg_pool).await?;
    let _ = AccountSSHKey::new(default_user.id, self.dosei_public_key.clone(), pg_pool).await;

    if let Some(accounts) = &self.accounts {
      let existing_accounts = Account::get_all(pg_pool).await?;
      let existing_non_dosei = existing_accounts
        .into_iter()
        .filter(|acc| acc.name != "dosei")
        .collect::<Vec<_>>();

      for account in accounts {
        if account.name == "dosei" {
          continue;
        }
        let db_account = Account::get_by_name(account.name.clone(), pg_pool).await?;
        if db_account.is_none() {
          let new_account = Account::new(&account.name, None, pg_pool).await?;
          if let Some(ssh_keys) = &account.ssh_keys {
            for ssh_key in ssh_keys {
              let _ = AccountSSHKey::new(new_account.id, ssh_key.clone(), pg_pool).await;
            }
          }
        }
      }

      let account_names: Vec<String> = accounts.iter().map(|acc| acc.name.clone()).collect();

      for existing_account in existing_non_dosei {
        if !account_names.contains(&existing_account.name) {
          existing_account.delete(pg_pool).await?;
        }
      }
    }

    let service = match Service::new("dosei", default_user.id, pg_pool).await {
      Ok(service) => service,
      Err(_) => Service::get_by_name("dosei".to_string(), pg_pool)
        .await?
        .unwrap(),
    };
    if Deployment::get_by_service_id(service.id, pg_pool)
      .await?
      .is_empty()
    {
      let _ = Deployment::new(service.id, service.owner_id, Some(80), Some(80), None, pg_pool).await;
    };

    // Ingress insert or Update
    match Ingress::get_by_service_id(service.id, pg_pool)
      .await?
      .first()
    {
      Some(ingress) => ingress.update_host(self.name.clone(), pg_pool).await?,
      None => Ingress::new(self.name.clone(), service.id, service.owner_id, pg_pool).await?,
    };

    // TODO: Hardcoded for testing, handle other cases
    Dashboard {
      name: self.name.clone().replace("api", "dashboard"),
    }
    .init(pg_pool)
    .await?;
    Ok(())
  }
}
