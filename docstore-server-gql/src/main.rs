use clap::Parser;
use snafu::{ResultExt, Snafu};

mod server;
mod settings;
mod utils;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Clap Error: {}", source))]
    Clap { source: clap::Error },
    #[snafu(display("Command Line Interface Error: {}", msg))]
    CLI { msg: String },
    #[snafu(display("Server Error: {}", source))]
    Server {
        #[snafu(backtrace)]
        source: server::Error,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = settings::Opts::try_parse().context(ClapSnafu)?;
    match opts.cmd {
        settings::Command::Run => server::run(&opts).await.context(ServerSnafu),
        settings::Command::Config => server::config(&opts).await.context(ServerSnafu),
    }
}
