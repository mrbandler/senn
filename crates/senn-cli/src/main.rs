//! Command-line interface of `senn`.

use clap::Parser;
use tracing_subscriber::EnvFilter;

/// Command-line interface of `senn`.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {}

/// Sets up logging to stderr (filtered by `RUST_LOG`) and parses the arguments.
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    tracing::debug!(?cli, "parsed arguments");
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
