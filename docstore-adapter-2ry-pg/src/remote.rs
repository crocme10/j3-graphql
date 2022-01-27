use snafu::{ResultExt, Snafu};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

use super::PostgresqlStorageConfig;
use docstore_domain::ports::secondary::remote::Error as RemoteError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Invalid Postgresql URL: {}, {}", details, source))]
    InvalidUrl {
        details: String,
        source: url::ParseError,
    },

    #[snafu(display("Postgresql Connection Error: {}", source))]
    PostgresqlConnection { source: sqlx::Error },

    /// Invalid Version Requirements
    #[snafu(display("Invalid Version Requirement Specification {}: {}", details, source))]
    VersionRequirementInvalid {
        details: String,
        source: semver::Error,
    },
}

impl From<Error> for RemoteError {
    fn from(error: Error) -> Self {
        RemoteError::Connection {
            source: Box::new(error),
        }
    }
}

pub async fn connection_pool(config: &PostgresqlStorageConfig) -> Result<PgPool, RemoteError> {
    // TODO Need to add checks
    PgPoolOptions::new()
        // Missing connection options
        .connect_timeout(Duration::from_millis(config.timeout))
        .connect(config.url.as_str())
        .await
        .context(PostgresqlConnection)
        .map_err(Into::into)
}

pub async fn connection_test_pool() -> Result<PgPool, RemoteError> {
    let config = PostgresqlStorageConfig::default_testing();
    connection_pool(&config).await
}
