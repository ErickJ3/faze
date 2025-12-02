use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "faze")]
#[command(about = "Local-first observability for developers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the collector and web UI
    Serve {
        /// HTTP/UI port
        #[arg(short, long, default_value = "7070")]
        port: u16,

        /// gRPC collector port
        #[arg(long, default_value = "4317")]
        grpc_port: u16,

        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Query traces
    Traces {
        #[arg(long)]
        slow: bool,

        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Query logs
    Logs {
        #[arg(long)]
        service: Option<String>,

        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Clean database
    Clean {
        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,

        /// Clean all databases
        #[arg(long)]
        all: bool,
    },

    /// Show DB information
    Info,

    /// Open TUI
    Tui,
}
