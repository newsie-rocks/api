//! Configuration  

use std::{net::SocketAddr, str::FromStr};

use config::Config;
use serde::Deserialize;
use tokio::sync::OnceCell;

/// Application configuration global instance
static APP_CONFIG: OnceCell<AppConfig> = OnceCell::const_new();

/// Application configuration
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Server config
    pub server: ServerConfig,
    /// Database config
    pub db: DbConfig,
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
}

impl AppConfig {
    /// Loads a configuration from environment variables and/or a config file
    pub async fn load() -> &'static Self {
        APP_CONFIG
            .get_or_init(|| async {
                let config = Config::builder()
                    .add_source(
                        config::Environment::with_prefix("APP")
                            .try_parsing(false)
                            .separator("_")
                            .list_separator(" "),
                    )
                    // .add_source(config::File::with_name("config"))
                    .build()
                    .unwrap();

                config.try_deserialize::<AppConfig>().unwrap()
            })
            .await
    }
}

/// Server configuration
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    /// jwt secret
    pub secret: String,
}

/// Database configuration
#[derive(Debug, Deserialize, Default)]
pub struct DbConfig {
    /// URL connection string
    pub url: String,
}

impl DbConfig {
    /// Creates a new [deadpool_postgres::Pool]
    pub fn pool(&self) -> deadpool_postgres::Pool {
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

/// Trace configuration
#[derive(Debug, Deserialize)]
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
        let db_url = std::env::var("APP_DB_URL").unwrap();
        let trace_stdout = std::env::var("APP_TRACE_STDOUT").unwrap();
        let trace_filter = std::env::var("APP_TRACE_FILTER").unwrap();
        assert_eq!(cfg.server.host, server_host);
        assert_eq!(cfg.server.port.to_string(), server_port);
        assert_eq!(cfg.auth.secret, auth_secret);
        assert_eq!(cfg.db.url, db_url);
        // NB:  trace.stdout is a bool, so .to_string() might fail depending on the APP_TRACE_STDOUT value
        assert_eq!(cfg.trace.stdout.to_string(), trace_stdout);
        assert_eq!(cfg.trace.filter.to_string(), trace_filter);
    }

    #[tokio::test]
    async fn test_db_conn() {
        let cfg = AppConfig::load().await;

        let db_pool = cfg.db.pool();
        let db_client = db_pool.get().await.unwrap();
        let rows = db_client.query("SELECT 1", &[]).await.unwrap();

        assert_eq!(rows.len(), 1);
    }
}
