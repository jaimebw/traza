mod web;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use chrono::Local;
use clap::Parser;
use dirs::data_local_dir;
use rusqlite::{params, Connection};

#[derive(Parser)]
#[command(version = "0.1.0", about = "traza - FPGA(or whatever you want) build logger", long_about = "traza is a simple utility that logs build outputs to a SQLite database. It captures stdin and associates it with a project name, timestamp, and optional tags for easy retrieval later.")]
struct Args {
    /// Name of the project to tag the log
    #[arg(long)]
    project: String,

    /// Optional tags for the log (comma-separated or multiple --tag)
    #[arg(long, value_delimiter = ',')]
    tag: Vec<String>,
}

fn get_db_path() -> PathBuf {
    let mut path = data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("traza");
    fs::create_dir_all(&path).expect("Failed to create data directory");
    path.push("logs.db");
        
    if cfg!(debug_assertions) {
        println!("🔍 Debug: Database path: {}", path.display());
    }
    
    path
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().len() == 1 {
        let db_path = get_db_path();
        return web::open_latest_log(&db_path);
    }
    let args = Args::parse();
    let db_path = get_db_path();

    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            tags TEXT,
            log TEXT NOT NULL
        )",
        [],
    )?;

    // Read full stdin into buffer
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read stdin");

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tags_csv = args.tag.join(",");

    conn.execute(
        "INSERT INTO logs (project, timestamp, tags, log) VALUES (?1, ?2, ?3, ?4)",
        params![args.project, timestamp, tags_csv, buffer],
    )?;

    println!(
        "✅ Log saved for project '{}' with tags [{}]",
        args.project, tags_csv
    );

    Ok(())
}

