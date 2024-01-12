use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
  pub id: Uuid,
  pub name: String,
  pub service_type: ServiceType,
  pub project_id: Option<Uuid>,
  pub storage_id: Option<Uuid>,
  pub deployment_id: Option<String>,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Domain {
  pub fn new(mut domain: Domain) -> Domain {
    domain.name = domain.name.to_lowercase();
    domain
  }
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "service_type", rename_all = "lowercase")]
pub enum ServiceType {
  Queued,
  Building,
  Error,
  Canceled,
  Ready,
}
