pub mod dosei {
    pub mod cron_job {
        include!(concat!(env!("OUT_DIR"), "/dosei.cron_job.rs"));
    }
}
