use snafu::Snafu;

use petstore_client_gql::Error as APIError;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("GraphQL API Error: {}", source))]
    GraphqlAPI { source: APIError },

    #[snafu(display("Environment Variable Error: {} ({})", details, source))]
    EnvironmentVariable {
        details: String,
        source: std::env::VarError,
    },

    #[snafu(display("Miscellaneous Error: {}", details))]
    Miscellaneous { details: String },

    #[snafu(display("Invalid URL: {} {}", details, source))]
    Url {
        details: String,
        source: url::ParseError,
    },
}
