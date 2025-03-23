use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use rusqlite::{Connection, Row};
use webbrowser;

pub fn open_latest_log(db_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, project, timestamp, tags, log
         FROM logs
         ORDER BY id DESC
         LIMIT 1",
    )?;

    let log = stmt.query_row([], |row: &Row| {
        Ok(LogEntry {
            id: row.get(0)?,
            project: row.get(1)?,
            timestamp: row.get(2)?,
            tags: row.get(3)?,
            log: row.get(4)?,
        })
    })?;

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Traza Log #{}</title>
    <style>
        body {{ font-family: monospace; background: #f4f4f4; padding: 2em; }}
        pre {{ background: #222; color: #0f0; padding: 1em; overflow-x: auto; }}
        h1, h2 {{ font-family: sans-serif; }}
    </style>
</head>
<body>
    <h1>Traza Log #{}</h1>
    <h2>Project: {}</h2>
    <h3>Tags: {}</h3>
    <h4>Timestamp: {}</h4>
    <pre>{}</pre>
</body>
</html>
"#,
        log.id,
        log.id,
        log.project,
        log.tags,
        log.timestamp,
        html_escape::encode_safe(&log.log)
    );

    let out_path = std::env::temp_dir().join("traza_latest_log.html");
    let mut file = File::create(&out_path)?;
    file.write_all(html.as_bytes())?;

    webbrowser::open(out_path.to_str().unwrap())?;

    Ok(())
}

struct LogEntry {
    id: i64,
    project: String,
    timestamp: String,
    tags: String,
    log: String,
}

