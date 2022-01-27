use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};
use docstore_adapter_1ry_gql as graphql;
use docstore_adapter_2ry_pg as postgresql;
use docstore_domain::model::error::Error as ModelError;
use http::StatusCode;
use snafu::{ResultExt, Snafu};
use std::convert::Infallible;
use std::net::ToSocketAddrs;
use tracing::instrument;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use warp::{http::Method, http::Response as HttpResponse, Filter, Rejection};

use super::settings::{Error as SettingsError, Opts, Settings};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Store Error: {}", source))]
    Store { source: postgresql::Error },

    #[snafu(display("Model Error: {}", source))]
    Model { source: ModelError },

    #[snafu(display("Could not generate settings: {}", source))]
    SettingsProcessing { source: SettingsError },

    #[snafu(display("Socket Addr Error with host {} / port {}: {}", host, port, source))]
    SockAddr {
        host: String,
        port: u16,
        source: std::io::Error,
    },

    #[snafu(display("Addr Resolution Error {}", msg))]
    AddrResolution { msg: String },

    #[snafu(display("Could not init log file: {}", source))]
    InitLog { source: std::io::Error },
}

#[allow(clippy::needless_lifetimes)]
pub async fn run(opts: &Opts) -> Result<(), Error> {
    let settings = Settings::new(opts).context(SettingsProcessing)?;
    LogTracer::init().expect("Unable to setup log tracer!");

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,gql=info".to_owned());

    // following code mostly from https://betterprogramming.pub/production-grade-logging-in-rust-applications-2c7fffd108a6
    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();

    // tracing_appender::non_blocking()
    let (non_blocking, _guard) = {
        if settings.logging.path.is_dir() {
            let file_appender = tracing_appender::rolling::daily(&settings.logging.path, "gql.log");

            tracing_appender::non_blocking(file_appender)
        } else {
            tracing_appender::non_blocking(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&settings.logging.path)
                    .context(InitLog)?,
            )
        }
    };

    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking);
    let subscriber = Registry::default()
        .with(EnvFilter::new(&filter))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).expect("tracing subscriber global default");

    run_server(settings).await
}

#[allow(clippy::needless_lifetimes)]
pub async fn config(opts: &Opts) -> Result<(), Error> {
    let settings = Settings::new(opts).context(SettingsProcessing)?;
    println!("{}", serde_json::to_string_pretty(&settings).unwrap());
    Ok(())
}

#[instrument(skip(settings))]
pub async fn run_server(settings: Settings) -> Result<(), Error> {
    let store = postgresql::PostgresqlStorage::new(&settings.postgresql)
        .await
        .context(Store)?;

    let service = Box::new(store);

    let schema = graphql::api::schema(service);

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (graphql::api::DocStoreSchema, async_graphql::Request)| async move {
            // let request_id = Uuid::new_v4();
            // let root_span = span!(parent: None, Level::INFO, "graphql request", %request_id);
            // let request = request.data(Tracing::default().parent_span(root_span));
            Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await))
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET, Method::POST])
        .allow_headers(vec!["content-type"]);

    let log = warp::log("backend");

    let routes = graphql_playground
        .or(graphql_post)
        .with(cors)
        .with(log)
        .recover(|err: Rejection| async move {
            if let Some(GraphQLBadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });

    let host = settings.service.host;
    let port = settings.service.port;
    let addr = (host.as_str(), port);
    let addr = addr
        .to_socket_addrs()
        .context(SockAddr { host, port })?
        .next()
        .ok_or(Error::AddrResolution {
            msg: String::from("Cannot resolve bragi addr."),
        })?;

    warp::serve(routes).run(addr).await;

    Ok(())
}
