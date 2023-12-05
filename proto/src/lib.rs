use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CronJob {
    pub id: String,
    pub schedule: String,
    pub entrypoint: String,
    pub deployment_id: String,
}
