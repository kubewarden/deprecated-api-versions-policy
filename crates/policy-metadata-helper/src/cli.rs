use clap::{Parser, Subcommand};

/// Find the most recent Kubernetes version mentioned inside of `versions.yml`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Path to the metadata.yml of the policy
    #[arg(short, long)]
    pub metadata_path: String,

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
    /// Propose the rules to be used inside of the policy metadata
    Build {},
    /// Check if the rules defined inside of the metadata are correct
    Check {},
}
