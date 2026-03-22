use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the DataSneaker client.
pub struct ClientConfig {
    pub server_url: String,
    pub app_key: Option<String>,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
    pub platform: Option<String>,
    pub os_version: Option<String>,
    pub flush_interval_ms: u64,
    pub max_batch_size: usize,
    pub max_queue_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".into(),
            app_key: None,
            user_id: None,
            device_id: None,
            app_version: None,
            platform: None,
            os_version: None,
            flush_interval_ms: 5000,
            max_batch_size: 50,
            max_queue_size: 1000,
        }
    }
}

/// Event payload sent to the DataSneaker server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub event_id: String,
    pub user_id: String,
    pub device_id: String,
    pub session_id: String,
    pub event_type: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}

/// Simplified event structure for the `track` method.
pub struct TrackEvent {
    pub event_type: String,
    pub screen_name: Option<String>,
    pub properties: Option<serde_json::Value>,
}

impl Default for TrackEvent {
    fn default() -> Self {
        Self {
            event_type: String::new(),
            screen_name: None,
            properties: None,
        }
    }
}

/// Server batch response.
#[derive(Debug, Deserialize)]
pub struct BatchResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<BatchData>,
}

#[derive(Debug, Deserialize)]
pub struct BatchData {
    pub processed: u64,
    pub failed: u64,
    pub processed_events: Vec<String>,
    pub failed_events: Vec<String>,
}

/// Server error response.
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error: Option<String>,
}

/// Server health response.
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub data: Option<HealthData>,
}

#[derive(Debug, Deserialize)]
pub struct HealthData {
    pub status: String,
    pub clickhouse_status: String,
    pub timestamp: i64,
    pub service: String,
    pub version: String,
}

/// Server stats response.
#[derive(Debug, Deserialize)]
pub struct StatsResponse {
    pub status: String,
    pub data: Option<StatsData>,
}

#[derive(Debug, Deserialize)]
pub struct StatsData {
    pub current_window: HashMap<String, i64>,
    pub timestamp: i64,
    pub historical_stats: Option<Vec<HistoricalStat>>,
}

#[derive(Debug, Deserialize)]
pub struct HistoricalStat {
    pub window_start: String,
    pub event_counts: HashMap<String, i64>,
}
