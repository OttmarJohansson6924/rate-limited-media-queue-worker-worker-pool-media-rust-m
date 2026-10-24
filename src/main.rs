use media_queue_worker::{decide, JobDecision, MediaJob};

fn main() {
    let job = MediaJob {
        asset_id: "asset-17".into(),
        creator_id: "creator-4".into(),
        delivery_url: "https://creator.test/deliver".into(),
    };
    match decide(job, 0, 2) {
        JobDecision::Process(job) => println!("processed {} for {}", job.asset_id, job.creator_id),
        JobDecision::Defer { asset_id, retry_after_seconds } => {
            println!("deferred {asset_id}; retry in {retry_after_seconds}s")
        }
    }
}
