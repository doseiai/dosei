pub trait ProtoChannel {
    const PROTO_ID: u8;
}

pub mod node_info {
    include!(concat!(env!("OUT_DIR"), "/dosei.cluster.rs"));
}

impl ProtoChannel for node_info::NodeInfo {
    const PROTO_ID: u8 = 0x00;
}

pub mod cron_job {
    include!(concat!(env!("OUT_DIR"), "/dosei.cron_job.rs"));
}

impl ProtoChannel for cron_job::CronJob {
    const PROTO_ID: u8 = 0x01;
}
