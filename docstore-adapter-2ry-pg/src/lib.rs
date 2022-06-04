use docstore_domain::model::error::Error as ModelError;
use docstore_domain::ports::secondary::remote::Error as RemoteError;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use snafu::Snafu;
use sqlx::postgres::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

pub mod remote;
pub mod storage;
pub mod utils;

/// An error type used to provide some context
/// on the sqlx error.
#[derive(Debug, Snafu)]
pub enum Error {
    /// The requested entity does not exist
    #[snafu(display("Connection"))]
    Connection { source: RemoteError },

    /// The requested entity does not exist
    #[snafu(display("Entity does not exist"))]
    NotFound,

    /// The operation violates a uniqueness constraint
    #[snafu(display("Operation violates uniqueness constraint: {}", details))]
    UniqueViolation { details: String },

    /// The requested operation violates the data model
    #[snafu(display("Operation violates model: {}", details))]
    ModelViolation { details: String },

    /// The requested operation violates the data model
    #[snafu(display("UnHandled Error: {}", source))]
    UnHandledError { source: sqlx::Error },
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Error::NotFound,
            sqlx::Error::Database(db_err) => Error::UnHandledError {
                source: sqlx::Error::Database(db_err),
            },
            _ => Error::UnHandledError { source: e },
        }
    }
}

impl From<Error> for ModelError {
    fn from(e: Error) -> Self {
        ModelError::Storage {
            source: Box::new(e),
        }
    }
}
#[derive(Debug)]
pub struct PostgresqlStorage {
    pub pool: Arc<PgPool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PostgresqlStorageConfig {
    pub url: Url,
    pub timeout: u64,
}

impl PostgresqlStorage {
    pub async fn new(config: &PostgresqlStorageConfig) -> Result<Self, Error> {
        let pool = remote::connection_pool(config)
            .await
            .context(ConnectionSnafu)?;
        Ok(PostgresqlStorage {
            pool: Arc::new(pool),
        })
    }
}

impl Default for PostgresqlStorageConfig {
    fn default() -> Self {
        let config_dir = PathBuf::from("config");
        let config =
            utils::config::config_from(config_dir.as_path(), &["postgresql"], None, None, vec![]);

        config
            .unwrap_or_else(|_| {
                panic!(
                    "cannot build default configuration for PostgreSQL from {}",
                    config_dir.display(),
                )
            })
            .get("postgresql")
            .unwrap_or_else(|_| {
                panic!(
                    "cannot extract 'postgresql' section from configuration {}",
                    config_dir.display(),
                )
            })
    }
}

impl PostgresqlStorageConfig {
    pub fn default_testing() -> Self {
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");

        let config = utils::config::config_from(
            config_dir.as_path(),
            &["postgresql"],
            "testing",
            "PG_TEST",
            vec![],
        );

        config
            .unwrap_or_else(|_| {
                panic!(
                    "cannot build testing configuration for PostgreSQL from {}",
                    config_dir.display(),
                )
            })
            .get("postgresql")
            .unwrap_or_else(|_| {
                panic!(
                    "cannot extract 'postgresql' section from testing configuration {}",
                    config_dir.display(),
                )
            })
    }
}

#[cfg(test)]
pub mod tests {

    use crate::remote::connection_test_pool;
    use crate::utils::docker;

    #[tokio::test]
    async fn should_connect_to_postgresql() {
        docker::initialize()
            .await
            .expect("postgresql docker initialization");

        let _client = connection_test_pool()
            .await
            .expect("Postgresql Connection Pool")
            .acquire()
            .await
            .expect("Postgresql Connection Established");
    }
}
