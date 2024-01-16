use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  pub id: Uuid,
  pub username: String,
  pub name: Option<String>,
  pub email: String,
  pub github: Option<Value>,
  pub gitlab: Option<Value>,
  pub bitbucket: Option<Value>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
