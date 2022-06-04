use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::env;
use std::path::PathBuf;

use docstore_adapter_2ry_pg::PostgresqlStorageConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const ENV_VAR_PREFIX: &str = "GQL";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Arg Match Error: {}", msg))]
    ArgMatch { msg: String },
    #[snafu(display("Arg Missing Error: {}", msg))]
    ArgMissing { msg: String },
    #[snafu(display("Env Var Missing Error: {} [{}]", msg, source))]
    EnvVarMissing { msg: String, source: env::VarError },
    #[snafu(display("Config Merge Error: {} [{}]", msg, source))]
    ConfigMerge {
        msg: String,
        source: config::ConfigError,
    },
    #[snafu(display("Config Value Error: {} [{}]", msg, source))]
    ConfigValue {
        msg: String,
        source: std::num::TryFromIntError,
    },
    #[snafu(display("Config Value Error: {} [{}]", msg, source))]
    ConfigParse {
        msg: String,
        source: std::num::ParseIntError,
    },

    #[snafu(display("Config Compilation Error: {}", source))]
    ConfigCompilation { source: crate::utils::config::Error },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Host on which we expose gql. Example: 'http://localhost', '0.0.0.0'
    pub host: String,
    /// Port on which we expose gql.
    pub port: u16,
    /// Used on POST request to set an upper limit on the size of the body (in bytes)
    pub content_length_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub mode: String,
    pub logging: Logging,
    pub postgresql: PostgresqlStorageConfig,
    pub service: Service,
}

#[derive(Debug, clap::Parser)]
#[clap(
    name = "gql",
    about = "GraphQL API for querying Postgresql Docstore Storage",
    version = VERSION,
    author = AUTHORS
    )]
pub struct Opts {
    /// Defines the config directory
    ///
    /// This directory must contain 'elasticsearch' and 'osm2mimir' subdirectories.
    #[clap(parse(from_os_str), short = 'c', long = "config-dir")]
    pub config_dir: PathBuf,

    /// Defines the run mode in {testing, dev, prod, ...}
    ///
    /// If no run mode is provided, a default behavior will be used.
    #[clap(short = 'm', long = "run-mode")]
    pub run_mode: Option<String>,

    /// Override settings values using key=value
    #[clap(
        short = 'o',
        long = "override",
        multiple_values = false,
        multiple_occurrences = true
    )]
    pub overrides: Vec<String>,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, clap::Parser)]
pub enum Command {
    /// Execute osm2mimir with the given configuration
    Run,
    /// Prints osm2mimir's configuration
    Config,
}

impl Settings {
    pub fn new(opts: &Opts) -> Result<Self, Error> {
        crate::utils::config::config_from(
            opts.config_dir.as_ref(),
            &["gql", "postgresql", "logging"],
            opts.run_mode.as_deref(),
            ENV_VAR_PREFIX,
            opts.overrides.clone(),
        )
        .context(ConfigCompilationSnafu)?
        .try_into()
        .context(ConfigMergeSnafu {
            msg: "cannot merge bragi settings",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_ok_with_default_config_dir() {
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");
        let opts = Opts {
            config_dir,
            run_mode: Some(String::from("testing")),
            overrides: vec![],
            cmd: Command::Run,
        };
        let settings = Settings::new(&opts);
        assert!(
            settings.is_ok(),
            "Expected Ok, Got an Err: {}",
            settings.unwrap_err().to_string()
        );
        assert_eq!(settings.unwrap().mode, String::from("testing"));
    }

    #[test]
    fn should_override_postgresql_port_with_command_line() {
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");
        let opts = Opts {
            config_dir,
            run_mode: Some(String::from("testing")),
            overrides: vec![String::from("postgresql.port=9999")],
            cmd: Command::Run,
        };
        let settings = Settings::new(&opts);
        assert!(
            settings.is_ok(),
            "Expected Ok, Got an Err: {}",
            settings.unwrap_err().to_string()
        );
        assert_eq!(settings.unwrap().postgresql.url.port().unwrap(), 9999);
    }

    #[test]
    fn should_override_postgresql_port_environment_variable() {
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");
        std::env::set_var("GQL_POSTGRESQL_URL", "http://localhost:9999");
        let opts = Opts {
            config_dir,
            run_mode: Some(String::from("testing")),
            overrides: vec![],
            cmd: Command::Run,
        };
        let settings = Settings::new(&opts);
        assert!(
            settings.is_ok(),
            "Expected Ok, Got an Err: {}",
            settings.unwrap_err().to_string()
        );
        assert_eq!(settings.unwrap().postgresql.url.port().unwrap(), 9999);
    }
}
