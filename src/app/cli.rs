//! Command line application

mod commands;
mod models;

use clap::{Parser, Subcommand};
use colored::*;
use commands::{duplicates_command, remove_command};
use dialoguer::Input;

const MYLIO_VAULT_ROOT: &str = "/Volumes/SamsungT9/Mylio_22c15a/Mylio Pictures";

#[derive(Parser)]
#[command(
    name = "Deduper",
    version = "1.0",
    author = "Richard Lyon <richlyon@fastmail.com>",
    about = "An image deduplication application"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for duplicates
    Duplicates {
        #[arg(short, long, default_value_t = false)]
        reset: bool,
    },
    /// Remove duplicates
    Remove {},
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Duplicates { reset } => handle_duplicates(*reset),
        Commands::Remove {} => handle_remove(),
    }
}

fn handle_duplicates(reset: bool) {
    dotenv::dotenv().ok();
    let vault_root =
        std::env::var("MYLIO_VAULT_ROOT").unwrap_or_else(|_| MYLIO_VAULT_ROOT.to_string());

    let root: String = Input::new()
        .with_prompt(format!("{}", "Vault root?".green()))
        .default(vault_root.to_string())
        .interact_text()
        .unwrap();

    match duplicates_command(&root, reset) {
        Ok(fail_count) => {
            println!(
                "Duplicates have been saved to '{}'",
                "results/duplicates.json".green()
            );
            if fail_count > 0 {
                println!(
                    "{}",
                    format!(
                        "Failed to hash {} images. Check the logs for more information.",
                        fail_count
                    )
                    .red()
                );
            }
        }
        Err(e) => eprintln!("Failed to save duplicates: {}", e),
    }
}

fn handle_remove() {
    match remove_command() {
        Ok(_) => println!("Duplicates have been removed"),
        Err(e) => eprintln!("Failed to remove duplicates: {}", e),
    }
}
