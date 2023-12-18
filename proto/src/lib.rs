pub trait ProtoChannel {
  const PROTO_ID: u8;
}

pub mod ping {
  include!(concat!(env!("OUT_DIR"), "/dosei.cluster.rs"));
}

impl ProtoChannel for ping::Ping {
  // PING
  const PROTO_ID: u8 = 0x00;
  // Reserve for PONG
  //const RESPONSE_PROTO_ID: u8 = 0x01;
}

pub mod cron_job {
  include!(concat!(env!("OUT_DIR"), "/dosei.cron_job.rs"));
}

impl ProtoChannel for cron_job::CronJob {
  const PROTO_ID: u8 = 0x02;
}
