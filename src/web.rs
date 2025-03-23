use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use rusqlite::{Connection, Row};
use webbrowser;

pub fn open_log_browser(db_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, project, timestamp, tags, log, hash FROM logs ORDER BY id DESC LIMIT 100",
    )?;

    let logs: Vec<LogEntry> = stmt
        .query_map([], |row: &Row| {
            Ok(LogEntry {
                id: row.get(0)?,
                project: row.get(1)?,
                timestamp: row.get(2)?,
                tags: row.get(3)?,
                log: row.get(4)?,
                hash: row.get(5)?
            })
        })?
        .filter_map(Result::ok)
        .collect();

    let out_dir = std::env::temp_dir().join("traza_ui");
    create_dir_all(&out_dir)?;

    // Write each log as a fragment
    for log in &logs {
        let fragment_path = out_dir.join(format!("log_{}.html", log.id));
        let mut f = File::create(&fragment_path)?;
        writeln!(
            f,
            r#"<h2>Project: {}</h2><h4>{} — Tags: {}</h4><pre>{}</pre>"#,
            log.project,
            log.timestamp,
            log.tags,
            html_escape::encode_safe(&log.log)
        )?;
    }

    // Write index.html
    let index_path = out_dir.join("index.html");
    let mut index = File::create(&index_path)?;
    writeln!(
        index,
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Traza Log Viewer</title>
    <style>
        body {{ font-family: sans-serif; background: #f4f4f4; padding: 1em; }}
        #logs {{ display: flex; gap: 1em; }}
        #sidebar {{ width: 300px; max-height: 90vh; overflow-y: auto; background: #fff; border-right: 1px solid #ccc; padding: 1em; }}
        #content {{ flex: 1; padding: 1em; background: #111; color: #0f0; overflow-x: auto; }}
        button {{ display: block; width: 100%; margin-bottom: 0.5em; }}
        pre {{ white-space: pre-wrap; word-wrap: break-word; }}
    </style>
</head>
<body>
    <h1>📘 Traza Logs</h1>
    <div id="logs">
        <div id="sidebar">
            {buttons}
        </div>
        <div id="content"><em>Select a log from the left.</em></div>
    </div>

    <script>
        function loadLog(id) {{
            fetch('log_' + id + '.html')
                .then(res => res.text())
                .then(html => {{
                    document.getElementById('content').innerHTML = html;
                }});
        }}
    </script>
</body>
</html>
"#,
        buttons = logs
            .iter()
    .map(|log| {
        format!(
            r#"<button onclick="window.location.href='log_{}.html'">[{}]_{} {} ({})</button>"#,
            log.id, log.id,log.hash ,log.project, log.timestamp
        )
    })
    .collect::<Vec<_>>()
    .join("\n")
    )?;

    webbrowser::open(index_path.to_str().unwrap())?;
    Ok(())
}

struct LogEntry {
    id: i64,
    project: String,
    timestamp: String,
    tags: String,
    log: String,
    hash: String,
}

