pub mod cluster_node {
    include!(concat!(env!("OUT_DIR"), "/dosei.cluster_node.rs"));
}

pub mod cron_job {
    include!(concat!(env!("OUT_DIR"), "/dosei.cron_job.rs"));
}
