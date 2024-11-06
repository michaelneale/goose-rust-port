use anyhow::Result;
use clap::{Parser, Subcommand, CommandFactory};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show version information
    #[arg(short = 'V', long)]
    version: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start or manage sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    /// Manage toolkits
    Toolkit {
        #[command(subcommand)]
        command: ToolkitCommands,
    },
    /// Run a single-pass session with a message from a markdown input file
    Run {
        /// Path to message file (optional)
        message_file: Option<PathBuf>,
        /// Profile to use
        #[arg(long)]
        profile: Option<String>,
        /// Log level
        #[arg(long, default_value = "INFO")]
        log_level: String,
        /// Resume the last session if available
        #[arg(long)]
        resume_session: bool,
        /// Enable tracing
        #[arg(long)]
        tracing: bool,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Start a new goose session
    Start {
        /// Session name
        name: Option<String>,
        /// Profile to use
        #[arg(long)]
        profile: Option<String>,
        /// Plan file path
        #[arg(long)]
        plan: Option<PathBuf>,
        /// Log level
        #[arg(long, default_value = "INFO")]
        log_level: String,
        /// Enable tracing
        #[arg(long)]
        tracing: bool,
    },
    /// List goose sessions
    List,
    /// Resume an existing goose session
    Resume {
        /// Session name
        name: Option<String>,
        /// Profile to use
        #[arg(long)]
        profile: Option<String>,
        /// Log level
        #[arg(long, default_value = "INFO")]
        log_level: String,
    },
    /// Delete old goose sessions
    Clear {
        /// Keep this many entries
        #[arg(long, default_value = "3")]
        keep: u32,
    },
}

#[derive(Subcommand)]
enum ToolkitCommands {
    /// List available toolkits
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        print_version();
        return Ok(());
    }

    match cli.command {
        Some(Commands::Session { command }) => match command {
            SessionCommands::Start { name: _, profile: _, plan: _, log_level: _, tracing: _ } => {
                println!("Starting session...");
                // TODO: Implement session start
            }
            SessionCommands::List => {
                println!("Listing sessions...");
                // TODO: Implement session list
            }
            SessionCommands::Resume { name: _, profile: _, log_level: _ } => {
                println!("Resuming session...");
                // TODO: Implement session resume
            }
            SessionCommands::Clear { keep: _ } => {
                println!("Clearing old sessions...");
                // TODO: Implement session clear
            }
        },
        Some(Commands::Toolkit { command }) => match command {
            ToolkitCommands::List => {
                println!("Available toolkits:");
                // TODO: Implement toolkit list
            }
        },
        Some(Commands::Run { message_file: _, profile: _, log_level: _, resume_session: _, tracing: _ }) => {
            println!("Running single-pass session...");
            // TODO: Implement run command
        }
        None => {
            println!("{}", <Cli as CommandFactory>::command().render_help());
        }
    }

    Ok(())
}

fn print_version() {
    println!("{}: {}", "Rust-goose".green(), env!("CARGO_PKG_VERSION").cyan().bold());
    println!("{}:", "Plugins".green());
    // TODO: Implement plugin version listing
}