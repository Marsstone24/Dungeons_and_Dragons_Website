#![allow(unused)]

use std::io;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use serde::de;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

#[derive(Parser)]
#[command(name = "R-DB-Manager")]
#[command(about = "Rustified Mariadb Manager for my D&D Website", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: DDLCommands,
}

// Data Definition Language
#[derive(Subcommand)]
enum DDLCommands {
    CreateTable { name: String },
    Alter { name: String },
    Drop { name: String },
}

// Data Manipulation Language
#[derive(Subcommand)]
enum DMLCommands {
    Select,
    Insert,
    Update,
    Delete,
    Merge,
}

// Access Commands for Frontend
#[derive(Subcommand)]
enum AccessCommands {
    Read,
    Give,
    Get,
    Insert,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
        dotenv().ok();

        let cli = Cli::parse();
        let database_url = std::env::var("DB_URL").expect("You must add a DB_URL inside your .env file");

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        match cli.command {
            DDLCommands::CreateTable { name } => create_table(name).await,
            DDLCommands::Alter { name } => alter_table(name).await,
            DDLCommands::Drop { name } => drop_table(name).await,
        }
        Ok(())
}

async fn create_table(name: String) {
    println!("{}", name);
}

async fn alter_table(name: String) {
    println!("{} 2", name);
}

async fn drop_table(name: String) {
    println!("{} 3", name);
}
