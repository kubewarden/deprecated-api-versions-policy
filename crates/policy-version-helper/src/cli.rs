use clap::{Parser, Subcommand};

/// Find the most recent Kubernetes version mentioned inside of `versions.yml`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Path to the Cargo.toml of the policy
    #[arg(short, long)]
    pub manifest_path: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub(crate) fn new() -> Self {
        Self::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Propose the version to be used by the policy
    Build {},
    /// Check if the version used by the policy is the right one
    Check {},
}
