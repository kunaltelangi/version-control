use clap::{Parser, Subcommand};
use std::process;

mod commands;

use commands::*;

#[derive(Parser)]
#[command(name = "kvcs")]
#[command(about = "A Git-like version control system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { files: Vec<String> },
    Status,
    Commit { 
        #[arg(short, long)]
        message: String 
    },
    Log,
    /// Create or list branches
    Branch { name: Option<String> },
    Checkout { target: String },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => init::execute(),
        Commands::Add { files } => add::execute(files),
        Commands::Status => status::execute(),
        Commands::Commit { message } => commit::execute(message),
        Commands::Log => log::execute(),
        Commands::Branch { name } => match name {
            Some(branch_name) => commands::branch::create(branch_name),
            None => commands::branch::list(),
        },
        Commands::Checkout { target } => checkout::execute(target),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}