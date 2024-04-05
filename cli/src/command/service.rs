use crate::config::Config;
use chrono::{DateTime, Utc};
use clap::Command;
use serde::{Deserialize, Serialize};

pub fn sub_command() -> Command {
  Command::new("service")
    .about("Services commands")
    .subcommand_required(true)
    .subcommand(Command::new("list").about("List Services"))
}

pub fn list_services(config: &'static Config) {
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .get(format!("{}/projects", config.api_base_url))
    .bearer_auth(config.bearer_token())
    .send()
    .unwrap();
  if response.status().is_success() {
    let services = response.json::<Vec<Service>>().unwrap();

    let headers = vec!["Name", "Last updated"];
    let mut rows = vec![];
    for service in services {
      rows.push(vec![service.name, format_time_ago(service.updated_at)]);
    }
    print_table(headers, rows);
  }
}

fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
  let num_columns = headers.len();
  let mut column_widths = vec![0; num_columns];
  for (i, header) in headers.iter().enumerate() {
    column_widths[i] = header.len();
  }
  for row in &rows {
    for (i, item) in row.iter().enumerate() {
      column_widths[i] = column_widths[i].max(item.len());
    }
  }
  for (i, header) in headers.iter().enumerate() {
    print!("{:<width$}   ", header, width = column_widths[i]);
  }
  println!();

  for row in rows {
    for (i, item) in row.iter().enumerate() {
      print!("{:<width$}   ", item, width = column_widths[i]);
    }
    println!();
  }
}

fn format_time_ago(from_datetime: DateTime<Utc>) -> String {
  let now = Utc::now();
  let duration = now.signed_duration_since(from_datetime);

  if duration.num_days() >= 1 {
    format!("{}d", duration.num_days())
  } else if duration.num_hours() >= 1 {
    format!("{}h", duration.num_hours())
  } else if duration.num_minutes() >= 1 {
    format!("{}m", duration.num_minutes())
  } else if duration.num_seconds() >= 1 {
    format!("{}s", duration.num_seconds())
  } else {
    "just now".to_string()
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Service {
  name: String,
  updated_at: DateTime<Utc>,
}
