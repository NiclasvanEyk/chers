use std::fmt;

use redis::{Client, aio::ConnectionManager};

#[derive(Debug)]
pub enum ConnectionError {
    NoUrl(String),
    FailedToConnect(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::NoUrl(name) => write!(
                f,
                "Failed to find a redis URL. Either use REDIS_URL or {name} to set it!"
            ),
            ConnectionError::FailedToConnect(error) => {
                write!(f, "Failed to connect to Redis: {error}")
            }
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Resolve a Redis URL from `{prefix}_REDIS_URL` env var, falling back to `REDIS_URL`.
pub fn resolve_url(prefix: &str) -> Result<String, ConnectionError> {
    let var_name = format!("{prefix}_REDIS_URL");
    std::env::var_os(&var_name)
        .or_else(|| std::env::var_os("REDIS_URL"))
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| ConnectionError::NoUrl(var_name))
}

/// Create a [`Client`] and [`ConnectionManager`] from a Redis URL.
pub async fn connect(url: &str) -> Result<(Client, ConnectionManager), ConnectionError> {
    let client = Client::open(url).map_err(|e| ConnectionError::FailedToConnect(e.into()))?;
    let conn = ConnectionManager::new(client.clone())
        .await
        .map_err(|e| ConnectionError::FailedToConnect(e.into()))?;
    Ok((client, conn))
}

/// Resolve URL from env and connect in one step.
pub async fn from_env(prefix: &str) -> Result<(Client, ConnectionManager), ConnectionError> {
    let url = resolve_url(prefix)?;
    connect(&url).await
}
