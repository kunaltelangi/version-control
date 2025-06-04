use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "vcs")]
#[command(about = "A custom version control system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { file: String },
    Commit { message: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run().await?,
        Commands::Add { file } => commands::add::run(file).await?,
        Commands::Commit { message } => commands::commit::run(message).await?,
    }

    Ok(())
}
