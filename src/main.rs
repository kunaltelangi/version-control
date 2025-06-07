use clap::{Parser, Subcommand};
use std::process;

mod commands;

use commands::*;

#[derive(Parser)]
#[command(name = "kvcs")]
#[command(about = "A Git-like version control system")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { 
        files: Vec<String>,
        #[arg(short, long)]
        all: bool,
    },
    Status,
    Commit { 
        #[arg(short, long)]
        message: String,
        #[arg(short = 'a', long)]
        all: bool, 
    },
    Log {
        #[arg(short, long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        oneline: bool,
    },
    Branch { 
        name: Option<String>,
        #[arg(short, long)]
        delete: bool,
    },
    Checkout { 
        target: String,
        #[arg(short, long)]
        create: bool,
    },
    Diff {
        #[arg(long)]
        cached: bool, 
        files: Vec<String>,
    },
    Merge { 
        branch: String,
        #[arg(long)]
        no_ff: bool, 
    },
    Reset {
        #[arg(long)]
        hard: bool,
        #[arg(long)]
        soft: bool,
        commit: Option<String>,
    },
    Stash {
        #[command(subcommand)]
        command: Option<StashCommands>,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Info,
}

#[derive(Subcommand)]
enum StashCommands {
    Push { 
        #[arg(short, long)]
        message: Option<String> 
    },
    Pop,
    List,
    Show { index: Option<usize> },
    Drop { index: usize },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Set { key: String, value: String },
    Get { key: String },
    List,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => init::execute(),
        Commands::Add { files, all } => add::execute(files, all),
        Commands::Status => status::execute(),
        Commands::Commit { message, all } => commit::execute(message, all),
        Commands::Log { limit, oneline } => log::execute(limit, oneline),
        Commands::Branch { name, delete } => match name {
            Some(branch_name) => {
                if delete {
                    branch::delete(branch_name)
                } else {
                    branch::create(branch_name)
                }
            },
            None => branch::list(),
        },
        Commands::Checkout { target, create } => checkout::execute(target, create),
        Commands::Diff { cached, files } => diff::execute(cached, files),
        Commands::Merge { branch, no_ff } => merge::execute(branch, no_ff),
        Commands::Reset { hard, soft, commit } => reset::execute(hard, soft, commit),
        Commands::Stash { command } => match command {
            Some(StashCommands::Push { message }) => stash::push(message),
            Some(StashCommands::Pop) => stash::pop(),
            Some(StashCommands::List) => stash::list(),
            Some(StashCommands::Show { index }) => stash::show(index),
            Some(StashCommands::Drop { index }) => stash::drop(index),
            None => stash::push(None),
        },
        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value } => config::set(key, value),
            ConfigCommands::Get { key } => config::get(key),
            ConfigCommands::List => config::list(),
        },
        Commands::Info => info::execute(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}