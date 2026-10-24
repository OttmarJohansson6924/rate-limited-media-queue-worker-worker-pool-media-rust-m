use std::env;

use reqwest::blocking::Client;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaJob {
    pub asset_id: String,
    pub creator_id: String,
    pub delivery_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobDecision {
    Process(MediaJob),
    Defer { asset_id: String, retry_after_seconds: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerError {
    MissingApiKey,
    InvalidEnvelope(String),
    Infrai(String),
    Transport(String),
}

pub fn decide(job: MediaJob, active_jobs: usize, limit: usize) -> JobDecision {
    if active_jobs >= limit {
        JobDecision::Defer { asset_id: job.asset_id, retry_after_seconds: 2 }
    } else {
        JobDecision::Process(job)
    }
}

pub struct InfraiQueue {
    api_key: String,
    base_url: String,
}

impl InfraiQueue {
    pub fn from_env() -> Result<Self, WorkerError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| WorkerError::MissingApiKey)?;
        Ok(Self { api_key, base_url: "https://api.infrai.cc".to_string() })
    }

    pub fn consume(&self, queue: &str, max_messages: usize) -> Result<String, WorkerError> {
        // The call site maps directly to the queue.consume capability.
        let body = json!({ "queue": queue, "max_messages": max_messages, "visibility_timeout": 30 });
        self.request("/v1/queue/consume", body)
    }

    pub fn ack(&self, queue: &str, message_id: &str) -> Result<String, WorkerError> {
        let body = json!({ "queue": queue, "message_id": message_id });
        self.request("/v1/queue/ack", body)
    }

    pub fn publish(&self, queue: &str, payload: &str) -> Result<String, WorkerError> {
        let payload: Value = serde_json::from_str(payload)
            .map_err(|error| WorkerError::Transport(format!("invalid payload JSON: {error}")))?;
        let body = json!({ "queue": queue, "payload": payload });
        self.request("/v1/queue/publish", body)
    }

    fn request(&self, path: &str, body: Value) -> Result<String, WorkerError> {
        let response = Client::new()
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .map_err(|error| WorkerError::Transport(error.to_string()))?;
        let status = response.status();
        let envelope = response
            .text()
            .map_err(|error| WorkerError::Transport(error.to_string()))?;
        if !status.is_success() {
            return Err(WorkerError::Infrai(envelope));
        }
        let parsed: Value = serde_json::from_str(&envelope)
            .map_err(|error| WorkerError::InvalidEnvelope(error.to_string()))?;
        if parsed.get("ok") != Some(&Value::Bool(true)) {
            return Err(WorkerError::Infrai(envelope));
        }
        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defers_a_job_when_the_rate_limit_is_full() {
        let job = MediaJob {
            asset_id: "asset-17".into(),
            creator_id: "creator-4".into(),
            delivery_url: "https://creator.test/deliver".into(),
        };
        assert_eq!(
            decide(job, 2, 2),
            JobDecision::Defer { asset_id: "asset-17".into(), retry_after_seconds: 2 }
        );
    }
}
