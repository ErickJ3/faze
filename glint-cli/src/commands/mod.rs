pub mod clean;
pub mod info;
pub mod logs;
pub mod serve;
pub mod traces;
pub mod tui;

use crate::cli::{Cli, Commands};

pub async fn dispatch(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Serve {
            port,
            grpc_port,
            db_path,
        } => serve::run(port, grpc_port, db_path).await,
        Commands::Traces { slow, db_path } => traces::run(slow, db_path).await,
        Commands::Logs { service, db_path } => logs::run(service, db_path).await,
        Commands::Clean { db_path, all } => clean::run(db_path, all).await,
        Commands::Info => info::run().await,
        Commands::Tui => tui::run().await,
    }
}
