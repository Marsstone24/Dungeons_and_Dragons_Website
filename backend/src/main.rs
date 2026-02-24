#![allow(unused)]

use std::collections::HashSet;
use colored::*;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use serde::de;
use sqlx::{MySql, Pool, Row, mysql::MySqlPoolOptions};
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::option;
use futures::stream::{StreamExt, FuturesUnordered};

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
    Up { name: Option<String> },
    Down { name: Option<String> },
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
    let database_url =
        std::env::var("DB_URL").expect("You must add a DB_URL inside your .env file");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    match cli.command {
        AdminCommands::Up { name } => migrate_table(name, &pool).await?,
        AdminCommands::Down { name } => drop_table(name, &pool).await?,
    }
    Ok(())
}

async fn migrate_table(name: Option<String>, pool: &Pool<MySql>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(name) = name {
        let query = fs::read_to_string(format!("tables/{}.sql", name))?;
        println!("Importing table...");
        sqlx::query(query.as_str()).execute(pool).await?;
    } else {
        let rows = sqlx::query(ASK_FOR_TABLES).fetch_all(pool).await?;
        let existing_tables: HashSet<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("TABLE_NAME"))
            .collect();
        let all_tables_as_read_dir = fs::read_dir("tables/")?;

        let mut all_tables: Vec<String> = Vec::new();
        let mut skipped_files: Vec<OsString> = Vec::new();
        for result_entry in all_tables_as_read_dir {
            let entry = result_entry?;
            let path = entry.path();
            
            if path.extension().and_then(|e| e.to_str()) == Some("sql") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    all_tables.push(name.to_string());
                }
            }
        }

        let tables: Vec<String> = all_tables.into_iter().filter(|table| !existing_tables.contains(table)).collect();
        
        if tables.is_empty() {
            println!("{}", "All tables are already inserted!".blue());
            return Ok(())
        }

        for table in &tables {
            println!("- {}", table);
        }
        println!("Do you want to import this tables? yes/no [no]:");

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("failed to read input");

        let input = buffer.trim().to_ascii_lowercase();
        match input.chars().next().unwrap_or('n') {
            'y' | 'j' => (),
            _ => {
                println!("Aborting...");
                return Ok(())
            }
        }

        println!("Importing tables...");

        let mut insert_tasks  = FuturesUnordered::new();

        for table in tables {
            let query = fs::read_to_string(format!("tables/{}.sql", table))?;
            insert_tasks.push(async move {
                sqlx::query(query.as_str()).execute(pool).await?;
                println!("Table {} was successfully inserted", table);
                Ok::<(), Box<dyn std::error::Error>>(())
            });
        }

        while let Some(result) = insert_tasks.next().await {
            result?
        }

    }
    println!("Done!");
    Ok(())
}

async fn drop_table(name: Option<String>, pool: &Pool<MySql>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(name) = name {
        let query = format!("DROP TABLE IF EXISTS {};", name);
        println!("Dropping table...");
        sqlx::query(query.as_str()).execute(pool).await?;
    } else {
        let rows = sqlx::query(ASK_FOR_TABLES).fetch_all(pool).await?;
        let existing_tables: HashSet<String> = rows
            .iter()
            .map(|row| row.get::<String, _>("TABLE_NAME"))
            .collect();

        if existing_tables.is_empty() {
            println!("{}", "The database has no tables at the moment!".blue());
            return Ok(())
        }

        for table in &existing_tables {
            println!("- {}", table);
        }
        println!("Are you sure to drop all this tables? yes/no [no]:");

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("failed to read input");

        let input = buffer.trim().to_ascii_lowercase();
        match input.chars().next().unwrap_or('n') {
            'y' | 'j' => (),
            _ => {
                println!("Aborting...");
                return Ok(())
            }
        }

        println!("Dropping tables...");

        let mut drop_tasks = FuturesUnordered::new();

        for table in existing_tables {
            let query = format!("DROP TABLE {};", table);
            drop_tasks.push(async move {
                sqlx::query(query.as_str()).execute(pool).await?;
                println!("Table {} was successfully dropped", table);
                Ok::<(), Box<dyn std::error::Error>>(())
            });
        }

        while let Some(result) = drop_tasks.next().await {
            result?
        }
    }

    Ok(())
}
