use bollard::auth::DockerCredentials;
use bollard::service::{HostConfig, PortBinding};
use bollard::{
    container::{
        Config as BollardConfig, CreateContainerOptions, ListContainersOptions,
        StartContainerOptions,
    },
    errors::Error as BollardError,
    image::CreateImageOptions,
    Docker,
};
use docstore_domain::ports::secondary::remote::Error as RemoteError;
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
// use tracing::info;

use crate::remote;
use crate::utils;
use crate::PostgresqlStorageConfig;

pub async fn initialize() -> Result<(), Error> {
    initialize_with_param(true).await
}

/// Initializes a docker container for testing
/// It will see if a docker container is available with the default name
/// If there is no container, it will create one.
/// If there is already a container, and the parameter cleanup is true,
/// then all:
/// * the indices found on that Elasticsearch are wiped out.
/// * the tables found in Postgresql are wiped out.
/// Once the container is available, a connection is attempted, to make
/// sure subsequent calls to that backend will be successful.
pub async fn initialize_with_param(cleanup: bool) -> Result<(), Error> {
    let mut docker = DockerWrapper::new();
    let is_available = docker.is_container_available().await?;
    if !is_available {
        docker.create_container().await?;
    } else if cleanup {
        docker.cleanup().await?;
    }
    let is_available = docker.is_container_available().await?;
    if !is_available {
        return Err(Error::DockerContainerUnavailable {
            name: docker.docker_config.container.name,
        });
    }
    let config = PostgresqlStorageConfig::default_testing();
    let _client = remote::connection_pool(&config)
        .await
        .context(PostgresqlPoolConnectionFailed)?
        .acquire()
        .await
        .context(PostgresqlConnectionFailed)?;

    Ok(())
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Connection to docker socket: {}", source))]
    DockerConnectionFailed { source: BollardError },

    #[snafu(display("Connection to postgresql: {}", source))]
    PostgresqlPoolConnectionFailed { source: RemoteError },

    #[snafu(display("Connection to postgresql: {}", source))]
    PostgresqlConnectionFailed { source: sqlx::Error },

    #[snafu(display("docker version: {}", source))]
    Version { source: BollardError },

    #[snafu(display("url parsing error: {}", source))]
    UrlParse { source: url::ParseError },

    #[snafu(display("postgresql client error: {}", source))]
    PostgresqlClient { source: sqlx::Error },

    #[snafu(display("docker error: {}", source))]
    DockerEngine { source: BollardError },

    #[snafu(display("docker container {} is not available", name))]
    DockerContainerUnavailable { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerVersion {
    pub major: usize,
    pub minor: usize,
}

impl From<DockerVersion> for bollard::ClientVersion {
    fn from(version: DockerVersion) -> bollard::ClientVersion {
        bollard::ClientVersion {
            major_version: version.major,
            minor_version: version.minor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub credentials: RegistryCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub image: String,
    pub name: String,
    pub memory: i64,
    pub vars: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub registry: RegistryConfig,
    pub container: ContainerConfig,
    pub timeout: u64,
    pub version: DockerVersion,
    pub container_wait: u64,
    pub container_available: u64,
    pub container_cleanup: u64,
}

impl Default for DockerConfig {
    /// We retrieve the docker configuration from ./config/docker/default.
    fn default() -> Self {
        let config =
            utils::config::config_from(&PathBuf::from("config"), &["docker"], None, None, vec![]);

        config
            .expect("cannot build the configuration for testing from config")
            .get("docker")
            .expect("expected docker section in configuration from config")
    }
}

pub struct DockerWrapper {
    ports: Vec<(u32, u32)>, // list of ports to publish (host port, container port)
    docker_config: DockerConfig,
}

impl DockerConfig {
    pub fn default_testing() -> Self {
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config");

        let config = utils::config::config_from(
            config_dir.as_path(),
            &["docker"],
            "testing",
            "TEST",
            vec![],
        );

        config
            .unwrap_or_else(|_| {
                panic!(
                    "cannot build the configuration for testing from {}",
                    config_dir.display(),
                )
            })
            .get("docker")
            .unwrap_or_else(|_| {
                panic!(
                    "expected docker section in configuration from {}",
                    config_dir.display(),
                )
            })
    }
    pub fn connect(&self) -> Result<Docker, Error> {
        Docker::connect_with_unix(
            "unix:///var/run/docker.sock",
            self.timeout,
            &self.version.clone().into(),
        )
        .context(DockerConnectionFailed)
    }
}

impl Default for DockerWrapper {
    fn default() -> Self {
        let postgresql_config = PostgresqlStorageConfig::default_testing();
        let docker_config = DockerConfig::default_testing();

        let port = postgresql_config
            .url
            .port()
            .expect("expected port in postgresql url");

        DockerWrapper {
            ports: vec![(port.into(), 5432)], // 5432 is the default pg port
            docker_config,
        }
    }
}

impl DockerWrapper {
    pub fn new() -> DockerWrapper {
        DockerWrapper::default()
    }

    // Returns true if the container self.docker_config.container.name is running
    pub async fn is_container_available(&mut self) -> Result<bool, Error> {
        let docker = self.docker_config.connect()?;

        let docker = &docker.negotiate_version().await.context(Version)?;

        docker.version().await.context(Version)?;

        let mut filters = HashMap::new();
        filters.insert("name", vec![self.docker_config.container.name.as_str()]);

        let options = Some(ListContainersOptions {
            all: false, // only running containers
            filters,
            ..Default::default()
        });

        let containers = docker
            .list_containers(options)
            .await
            .context(DockerEngine)?;

        Ok(!containers.is_empty())
    }

    // If the container is already created, then start it.
    // If it is not created, then create it and start it.
    pub async fn create_container(&mut self) -> Result<(), Error> {
        let docker = self.docker_config.connect()?;

        let docker = docker.negotiate_version().await.context(Version)?;

        let _ = docker.version().await.context(Version);

        let mut filters = HashMap::new();
        filters.insert("name", vec![self.docker_config.container.name.as_str()]);

        let options = Some(ListContainersOptions {
            all: true, // only running containers
            filters,
            ..Default::default()
        });

        let containers = docker
            .list_containers(options)
            .await
            .context(DockerEngine)?;

        if containers.is_empty() {
            let options = CreateContainerOptions {
                name: &self.docker_config.container.name,
            };

            let mut port_bindings = HashMap::new();
            for (host_port, container_port) in self.ports.iter() {
                port_bindings.insert(
                    format!("{}/tcp", &container_port),
                    Some(vec![PortBinding {
                        host_ip: Some(String::from("0.0.0.0")),
                        host_port: Some(host_port.to_string()),
                    }]),
                );
            }

            let host_config = HostConfig {
                port_bindings: Some(port_bindings),
                memory: Some(self.docker_config.container.memory * 1024 * 1024),
                ..Default::default()
            };

            let mut exposed_ports = HashMap::new();
            self.ports.iter().for_each(|(_, container)| {
                let v: HashMap<(), ()> = HashMap::new();
                exposed_ports.insert(format!("{}/tcp", container), v);
            });

            let env = Some(self.docker_config.container.vars.clone()).and_then(|vars| {
                if vars.is_empty() {
                    None
                } else {
                    Some(vars)
                }
            });

            let credentials = DockerCredentials {
                username: Some(self.docker_config.registry.credentials.username.clone()),
                password: Some(self.docker_config.registry.credentials.password.clone()),
                ..Default::default()
            };

            let config = BollardConfig {
                image: Some(self.docker_config.container.image.clone()),
                exposed_ports: Some(exposed_ports),
                host_config: Some(host_config),
                env,
                ..Default::default()
            };

            docker
                .create_image(
                    Some(CreateImageOptions {
                        from_image: self.docker_config.container.image.clone(),
                        ..Default::default()
                    }),
                    None,
                    Some(credentials),
                )
                .try_collect::<Vec<_>>()
                .await
                .context(DockerEngine)?;

            let _ = docker
                .create_container(Some(options), config)
                .await
                .context(DockerEngine)?;

            sleep(Duration::from_millis(self.docker_config.container_wait)).await;
        }
        let _ = docker
            .start_container(
                &self.docker_config.container.name,
                None::<StartContainerOptions<String>>,
            )
            .await
            .context(DockerEngine)?;

        sleep(Duration::from_millis(
            self.docker_config.container_available,
        ))
        .await;

        Ok(())
    }

    /// This function cleans up the Elasticsearch
    async fn cleanup(&mut self) -> Result<(), Error> {
        let _pool = remote::connection_test_pool();

        /* FIXME Missing implementation */

        sleep(Duration::from_millis(self.docker_config.container_cleanup)).await;
        Ok(())
    }

    async fn _drop(&mut self) {
        if std::env::var("DONT_KILL_THE_WHALE") == Ok("1".to_string()) {
            println!(
                "the docker won't be stoped at the end, you can debug it.
                Note: ES has been mapped to the port 9242 in you localhost
                manually stop and rm the container mimirsbrunn_tests after debug"
            );
            return;
        }
        let docker = self
            .docker_config
            .connect()
            .expect("docker engine connection");

        let options = Some(bollard::container::StopContainerOptions { t: 0 });
        docker
            .stop_container(&self.docker_config.container.name, options)
            .await
            .expect("stop container");

        let options = Some(bollard::container::RemoveContainerOptions {
            force: true,
            ..Default::default()
        });

        let _res = docker
            .remove_container(&self.docker_config.container.name, options)
            .await
            .expect("remove container");
    }
}
