//! The `senn` command.

use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;
use senn::herd::Herd;
use senn::runner::System;
use tracing_subscriber::EnvFilter;

/// Git-native infrastructure on Nix and OpenTofu.
#[derive(Debug, Parser)]
#[command(name = "senn", version)]
struct Cli {
    /// Path to the herd repository.
    #[arg(long, global = true, default_value = ".")]
    herd: PathBuf,

    #[command(subcommand)]
    command: Command,
}

/// The `senn` subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Show what would change in the core root.
    Plan,

    /// Create, install and configure everything declared in the herd.
    Apply {
        /// Apply OpenTofu changes without asking.
        #[arg(long)]
        yes: bool,
    },

    /// Push host secrets and deploy NixOS configurations, without OpenTofu.
    Deploy {
        /// Only this host.
        host: Option<String>,
    },
}

impl Cli {
    /// Asks `question` on stderr and reads the answer from stdin. Only `y` means yes.
    fn ask(question: &str) -> senn::Result<bool> {
        eprint!("{question} [y/N] ");
        std::io::stderr().flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;

        Ok(answer.trim().eq_ignore_ascii_case("y"))
    }
}

/// Sets up logging to stderr (filtered by `RUST_LOG`), then runs the subcommand.
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    tracing::debug!(?cli, "parsed arguments");

    let path = std::fs::canonicalize(&cli.herd).into_diagnostic()?;
    let runner = System;
    let herd = Herd::load(&runner, &path)?;
    match cli.command {
        Command::Plan => herd.plan()?,
        Command::Apply { yes } => herd.apply(&|| Ok(yes || Cli::ask("Apply these changes?")?))?,
        Command::Deploy { host } => herd.deploy(host.as_deref())?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
