use crate::error::SdkError;
use crate::types::{ClientConfig, TrackEvent};

/// Blocking (synchronous) wrapper around the async DataSneaker client.
pub struct Client {
    runtime: tokio::runtime::Runtime,
    inner: Option<crate::client::Client>,
}

impl Client {
    /// Create a new blocking DataSneaker client.
    pub fn new(config: ClientConfig) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");

        let inner = runtime.block_on(async { crate::client::Client::new(config) });

        Self {
            runtime,
            inner: Some(inner),
        }
    }

    /// Track an event synchronously.
    pub fn track(&self, event: TrackEvent) -> Result<(), SdkError> {
        let inner = self.inner.as_ref().ok_or(SdkError::Shutdown)?;
        self.runtime.block_on(inner.track(event))
    }

    /// Manually flush all queued events.
    pub fn flush(&self) -> Result<(), SdkError> {
        let inner = self.inner.as_ref().ok_or(SdkError::Shutdown)?;
        self.runtime.block_on(inner.flush())
    }

    /// Update the user ID.
    pub fn set_user_id(&self, user_id: String) {
        if let Some(ref inner) = self.inner {
            self.runtime.block_on(inner.set_user_id(user_id));
        }
    }

    /// Shutdown the client and flush remaining events.
    pub fn shutdown(&mut self) -> Result<(), SdkError> {
        if let Some(inner) = self.inner.take() {
            self.runtime.block_on(inner.shutdown())?;
        }
        Ok(())
    }
}
