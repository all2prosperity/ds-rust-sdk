#[derive(thiserror::Error, Debug)]
pub enum SdkError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("queue full, event dropped")]
    QueueFull,

    #[error("server returned error: {status} {body}")]
    Server { status: u16, body: String },

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("client already shut down")]
    Shutdown,
}
