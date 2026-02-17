#![allow(unused)]

use std::io;
use std::fs;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use serde::de;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions, Row};

const ASK_FOR_TABLES: &str = "SELECT TABLE_NAME FROM information_schema.tables WHERE TABLE_SCHEMA = 'Dungeons_and_Dragons_DB'";

#[derive(Parser)]
#[command(name = "R-DB-Manager")]
#[command(about = "Rustified Mariadb Manager for my D&D Website", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: AdminCommands,
}

// Data Definition Language
#[derive(Subcommand)]
enum AdminCommands {
    Import {name: Option<String>},
    Alter { name: String },
    Drop { name: String },
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        dotenv().ok();

        let cli = Cli::parse();
        let database_url = std::env::var("DB_URL").expect("You must add a DB_URL inside your .env file");

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        match cli.command {
            AdminCommands::Import { name } => create_table(name, &pool).await?,
            AdminCommands::Alter { name } => alter_table(name).await?,
            AdminCommands::Drop { name } => drop_table(name).await?,
        }
        Ok(())
}

async fn create_table(name: Option<String>, pool: &Pool<MySql>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(name) = name {
        let query = fs::read_to_string(format!("tables/{}.sql", name))?;
        println!("Importing table...");
        sqlx::query(query.as_str()).execute(pool).await?;
    } else {
        let rows = sqlx::query(ASK_FOR_TABLES).fetch_all(pool).await?;
        let tables = rows.iter().map(|row| row.get::<String,_>("TABLE_NAME")).collect::<Vec<String>>();

        for table in &tables {
            println!("- {}", table);
        }
        println!("Do you want to import this {} tables? yes/no [no]:", &tables.len());
        
        let mut _buffer = String::new();
        io::stdin().read_line(&mut _buffer).expect("failed to read input");
        match _buffer.as_str().trim() {
            "yes" => println!(""),
            "no" => return Ok(()),
            _ => return Ok(())
        }
        println!("Importing tables...");
        // yellow Exchange the current for with a extern function which uses futures::stream::FutureUnordered to asnychronise all table creations to create them simultaniously
        for table in tables {
            let query = fs::read_to_string(format!("tables/{}.sql", table))?;
            sqlx::query(query.as_str()).execute(pool).await?;
        }
    }
    println!("Done!");
    Ok(())
}

async fn alter_table(name: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Not made this command yet!");
    Ok(())
}

async fn drop_table(name: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Not made this command yet!");
    Ok(())
}
