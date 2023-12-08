pub mod node_info {
    include!(concat!(env!("OUT_DIR"), "/dosei.cluster.rs"));
}

pub mod cron_job {
    include!(concat!(env!("OUT_DIR"), "/dosei.cron_job.rs"));
}
