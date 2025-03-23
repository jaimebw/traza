mod web;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use chrono::Local;
use clap::Parser;
use dirs::data_local_dir;
use rusqlite::{params, Connection};

use blake3;

#[derive(Parser)]
#[command(
    version = "0.1.0",
    about = "traza - FPGA (or whatever you want) build logger",
    long_about = "traza is a simple utility that logs build outputs to a SQLite database. It captures stdin and associates it with a project name, timestamp, and optional tags for easy retrieval later."
)]
struct Args {
    /// Name of the project to tag the log
    #[arg(long)]
    project: Option<String>,

    /// If db=delete, deletes all logs from the SQLite DB
    #[arg(long)]
    db: Option<String>,

    /// Get a list of the logs
    #[arg(long, action = clap::ArgAction::SetTrue)]
    list: bool,

    /// Export a log by hash prefix
    #[arg(long)]
    export: Option<String>,

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
    // No arguments: launch web viewer
    if env::args().len() == 1 {
        let db_path = get_db_path();
        return web::open_log_browser(&db_path);
    }

    let args = Args::parse();
    let db_path = get_db_path();

    // --db delete
    if let Some(db_action) = &args.db {
        if db_action == "delete" {
            let conn = Connection::open(&db_path)?;
            conn.execute("DELETE FROM logs", [])?;
            println!("🧹 All logs deleted from database");
            return Ok(());
        }
    }

    // --list
    if args.list {
        let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, hash, project, timestamp, tags FROM logs ORDER BY timestamp DESC"
        )?;
        let logs = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;

        println!("📋 Logs List:");
        println!(
            "{:<5} {:<8} {:<20} {:<20} {:<30}",
            "ID", "Hash", "Project", "Timestamp", "Tags"
        );
        println!("{}", "-".repeat(90));

        for log in logs {
            if let Ok((id, hash, project, timestamp, tags)) = log {
                let tags_display = tags.unwrap_or_default();
                println!(
                    "{:<5} {:<8} {:<20} {:<20} {:<30}",
                    id, hash, project, timestamp, tags_display
                );
            }
        }

        return Ok(());
    }

    // --export <hash>
    if let Some(prefix) = &args.export {
        let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare(
            "SELECT hash, project, timestamp, log FROM logs WHERE hash LIKE ?1 || '%' LIMIT 1"
        )?;

        let log_result = stmt.query_row([prefix], |row| {
            Ok((
                row.get::<_, String>(0)?, // hash
                row.get::<_, String>(1)?, // project
                row.get::<_, String>(2)?, // timestamp
                row.get::<_, String>(3)?, // log
            ))
        });

        match log_result {
            Ok((hash, project, timestamp, log)) => {
                let filename = format!(
                    "{}/traza_export_{}_{}.txt",
                    dirs::home_dir().unwrap().display(),
                    project,
                    hash
                );
                fs::write(
                    &filename,
                    format!("# Project: {}\n# Timestamp: {}\n\n{}", project, timestamp, log),
                )?;
                println!("📤 Exported log to: {}", filename);
            }
            Err(_) => {
                eprintln!("❌ No log found with hash starting with '{}'", prefix);
            }
        }

        return Ok(());
    }

    // Insert a new log entry: require --project
    let Some(project) = args.project else {
        eprintln!("❌ Missing required argument: --project");
        std::process::exit(1);
    };

    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hash TEXT NOT NULL,
            project TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            tags TEXT,
            log TEXT NOT NULL
        )",
        [],
    )?;

    // Read log from stdin
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read stdin");

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tags_csv = args.tag.join(",");

    // Generate hash (based on project + timestamp + log)
    let mut hasher = blake3::Hasher::new();
    hasher.update(project.as_bytes());
    hasher.update(timestamp.as_bytes());
    hasher.update(buffer.as_bytes());
    let hash = hasher.finalize().to_hex()[..6].to_string();

    conn.execute(
        "INSERT INTO logs (hash, project, timestamp, tags, log) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![hash, project, timestamp, tags_csv, buffer],
    )?;

    println!(
        "✅ Log saved for project '{}' with tags [{}] and hash [{}]",
        project, tags_csv, hash
    );

    Ok(())
}

