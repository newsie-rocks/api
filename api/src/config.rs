//! Configuration  

use std::{net::SocketAddr, str::FromStr};

use config::Config;
use qdrant_client::prelude::*;
use serde::Deserialize;

/// Application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// Server config
    pub server: ServerConfig,
    /// PostGreSQL config
    pub postgres: PostGresConfig,
    /// Qdrant config
    pub qdrant: QdrantConfig,
    /// Auth configuration
    pub auth: AuthConfig,
    /// Trace configuration
    pub trace: TraceConfig,
}

/// Application configuration error
#[derive(Debug, thiserror::Error)]
pub enum AppConfigError {
    /// Invalid server host configuration
    #[error("invalid server address")]
    InvalidServerHost(#[from] std::net::AddrParseError),
    /// Invalid qdrant config
    #[error("invalid qdrant config: {0}")]
    InvalidQdrantConfig(String),
}

impl AppConfig {
    /// Loads a configuration from the environment
    pub async fn load() -> Self {
        let config = Config::builder()
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(false)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()
            .unwrap();

        config.try_deserialize::<AppConfig>().unwrap()
    }
}

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Host
    pub host: String,
    /// Port
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3000,
        }
    }
}

impl ServerConfig {
    /// Returns the server [SocketAddr]
    pub fn addr(&self) -> Result<SocketAddr, AppConfigError> {
        let addr_str = self.host.to_string() + ":" + self.port.to_string().as_str();
        addr_str.parse::<SocketAddr>().map_err(|err| err.into())
    }
}

/// Auth configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    /// JWT secret
    pub secret: String,
}

/// Postgres DB configuration
#[derive(Debug, Deserialize, Clone)]
pub struct PostGresConfig {
    /// URL connection string
    pub url: String,
}

impl Default for PostGresConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://nick:enter@localhost:5432/newsie?connect_timeout=10".into(),
        }
    }
}

impl PostGresConfig {
    /// Creates a new [deadpool_postgres::Pool]
    pub fn new_pool(&self) -> deadpool_postgres::Pool {
        // set TLS config
        let tls = tokio_postgres::tls::NoTls;

        // create a [tokio_postgres::Config]
        let pg_config = tokio_postgres::Config::from_str(&self.url).unwrap();

        // set pool manager
        let mgr_config = deadpool_postgres::ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Fast,
        };
        let mgr = deadpool_postgres::Manager::from_config(pg_config, tls, mgr_config);

        // create the pool
        deadpool_postgres::Pool::builder(mgr)
            .max_size(100)
            .build()
            .unwrap()
    }
}

/// Qdrant configuration
#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    /// URL connection string
    pub url: String,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".into(),
        }
    }
}

impl QdrantConfig {
    /// Creates a new [QdrantClient]
    pub fn new_client(&self) -> QdrantClient {
        let config = QdrantClientConfig::from_url(&self.url);
        QdrantClient::new(Some(config)).unwrap()
    }
}

/// Trace configuration
#[derive(Debug, Deserialize, Clone)]
pub struct TraceConfig {
    /// Export traces to stdout
    pub stdout: bool,
    /// Trace filter
    pub filter: String,
}

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[tokio::test]
    async fn test_load_config() {
        let cfg = AppConfig::load().await;

        let server_host = std::env::var("APP_SERVER_HOST").unwrap();
        let server_port = std::env::var("APP_SERVER_PORT").unwrap();
        let auth_secret = std::env::var("APP_AUTH_SECRET").unwrap();
        let postgres_url = std::env::var("APP_POSTGRES_URL").unwrap();
        let qdrant_url = std::env::var("APP_QDRANT_URL").unwrap();
        let trace_stdout = std::env::var("APP_TRACE_STDOUT").unwrap();
        let trace_filter = std::env::var("APP_TRACE_FILTER").unwrap();
        assert_eq!(cfg.server.host, server_host);
        assert_eq!(cfg.server.port.to_string(), server_port);
        assert_eq!(cfg.auth.secret, auth_secret);
        assert_eq!(cfg.postgres.url, postgres_url);
        assert_eq!(cfg.qdrant.url, qdrant_url);
        // NB:  trace.stdout is a bool, so .to_string() might fail depending on the APP_TRACE_STDOUT value
        assert_eq!(cfg.trace.stdout.to_string(), trace_stdout);
        assert_eq!(cfg.trace.filter, trace_filter);
    }

    #[tokio::test]
    async fn test_postgres_conn() {
        let cfg = AppConfig::load().await;

        let postgres_pool = cfg.postgres.new_pool();
        let postgres_client = postgres_pool.get().await.unwrap();
        let rows = postgres_client.query("SELECT 1", &[]).await.unwrap();

        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn test_qdrant_conn() {
        let cfg = AppConfig::load().await;

        let qdrant_client = cfg.qdrant.new_client();
        qdrant_client.health_check().await.unwrap();
    }
}
