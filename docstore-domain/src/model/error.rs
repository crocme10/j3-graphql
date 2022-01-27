use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Storage Error: {}", source))]
    Storage { source: Box<dyn std::error::Error> },
}
