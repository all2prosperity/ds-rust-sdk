use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tokio::sync::{Mutex, Notify};
use tracing;

use crate::error::SdkError;
use crate::types::{ClientConfig, EventPayload, TrackEvent};

/// DataSneaker client for event tracking.
pub struct Client {
    config: ClientConfig,
    http_client: reqwest::Client,
    queue: Arc<Mutex<VecDeque<EventPayload>>>,
    device_id: String,
    session_id: String,
    user_id: Arc<Mutex<String>>,
    shutdown_notify: Arc<Notify>,
    flush_notify: Arc<Notify>,
    _background_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Client {
    /// Create a new DataSneaker client and start the background flush task.
    pub fn new(config: ClientConfig) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref key) = config.app_key {
            if let Ok(val) = HeaderValue::from_str(key) {
                default_headers.insert("X-App-Key", val);
            }
        }

        let http_client = reqwest::Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");

        let device_id = config
            .device_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let session_id = uuid::Uuid::new_v4().to_string();
        let user_id = Arc::new(Mutex::new(config.user_id.clone()));

        let queue: Arc<Mutex<VecDeque<EventPayload>>> = Arc::new(Mutex::new(VecDeque::new()));
        let shutdown_notify = Arc::new(Notify::new());
        let flush_notify = Arc::new(Notify::new());

        let handle = {
            let queue = Arc::clone(&queue);
            let shutdown = Arc::clone(&shutdown_notify);
            let flush = Arc::clone(&flush_notify);
            let url = format!("{}/api/v1/track/batch", config.server_url);
            let client = http_client.clone();
            let interval = Duration::from_millis(config.flush_interval_ms);

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(interval) => {}
                        _ = flush.notified() => {}
                        _ = shutdown.notified() => {
                            flush_queue(&client, &url, &queue).await;
                            return;
                        }
                    }
                    flush_queue(&client, &url, &queue).await;
                }
            })
        };

        Self {
            config,
            http_client,
            queue,
            device_id,
            session_id,
            user_id,
            shutdown_notify,
            flush_notify,
            _background_handle: Some(handle),
        }
    }

    /// Track an event. Returns an error if the queue is full.
    pub async fn track(&self, event: TrackEvent) -> Result<(), SdkError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let user_id = self.user_id.lock().await.clone();

        let payload = EventPayload {
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            device_id: self.device_id.clone(),
            session_id: self.session_id.clone(),
            event_type: event.event_type,
            timestamp: now,
            app_version: self.config.app_version.clone(),
            platform: self.config.platform.clone(),
            os_version: self.config.os_version.clone(),
            screen_name: event.screen_name,
            properties: event.properties,
        };

        let should_flush;
        {
            let mut q = self.queue.lock().await;
            if q.len() >= self.config.max_queue_size {
                return Err(SdkError::QueueFull);
            }
            q.push_back(payload);
            should_flush = q.len() >= self.config.max_batch_size;
        }

        if should_flush {
            self.flush_notify.notify_one();
        }

        Ok(())
    }

    /// Manually flush all queued events.
    pub async fn flush(&self) -> Result<(), SdkError> {
        let url = format!("{}/api/v1/track/batch", self.config.server_url);
        flush_queue(&self.http_client, &url, &self.queue).await;
        Ok(())
    }

    /// Update the user ID for subsequent events.
    pub async fn set_user_id(&self, user_id: String) {
        *self.user_id.lock().await = user_id;
    }

    /// Shutdown the client: stop the background task and flush remaining events.
    pub async fn shutdown(self) -> Result<(), SdkError> {
        self.shutdown_notify.notify_one();
        if let Some(handle) = self._background_handle {
            let _ = handle.await;
        }
        Ok(())
    }
}

async fn flush_queue(
    client: &reqwest::Client,
    url: &str,
    queue: &Arc<Mutex<VecDeque<EventPayload>>>,
) {
    let events: Vec<EventPayload> = {
        let mut q = queue.lock().await;
        if q.is_empty() {
            return;
        }
        q.drain(..).collect()
    };

    let count = events.len();
    match client.post(url).json(&events).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::debug!("flushed {} events", count);
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("flush failed: {} {}", status, body);
            // Put events back
            let mut q = queue.lock().await;
            for event in events.into_iter().rev() {
                q.push_front(event);
            }
        }
        Err(err) => {
            tracing::error!("flush error: {}", err);
            let mut q = queue.lock().await;
            for event in events.into_iter().rev() {
                q.push_front(event);
            }
        }
    }
}
