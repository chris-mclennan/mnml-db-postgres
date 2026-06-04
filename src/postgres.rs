//! tokio-postgres wrapper. v0.1 uses the **simple query protocol**
//! (text mode) — every value comes back as `Option<&str>` and we
//! render it verbatim. No per-type formatting, no type coercion.
//! v0.2 will move to the binary protocol with rich-type rendering.

use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls, SimpleQueryMessage};

/// Open a Postgres connection from a DSN. Spawns the connection
/// driver on the current tokio runtime; caller owns the resulting
/// `Client`.
pub async fn connect(dsn: &str) -> Result<Client> {
    let (client, connection) = tokio_postgres::connect(dsn, NoTls)
        .await
        .context("connecting to Postgres")?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("mnml-db-postgres: connection driver error: {e}");
        }
    });
    Ok(client)
}

/// Returned rows + column headers, ready for the TUI's table widget.
#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// Time the query took to complete (round-trip including row
    /// streaming).
    pub elapsed: std::time::Duration,
    /// Total rows returned by the server (may exceed `rows.len()`
    /// when the client-side cap truncates the view).
    pub server_row_count: usize,
    /// True when `rows.len() < server_row_count` (cap hit). UI shows
    /// a `truncated` chip in the results title.
    pub truncated: bool,
}

/// Run a query against `client`. Caps the materialized result at
/// `row_limit` to keep an accidental `SELECT *` from a 10M-row table
/// from buffering forever. NULLs render as the literal `NULL`.
pub async fn run_query(client: &Client, sql: &str, row_limit: u32) -> Result<QueryResult> {
    let start = std::time::Instant::now();
    let messages = client.simple_query(sql).await.context("running query")?;
    let elapsed = start.elapsed();

    let mut columns: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut server_row_count = 0usize;
    let mut truncated = false;
    for msg in messages {
        match msg {
            SimpleQueryMessage::RowDescription(cols) => {
                columns = cols.iter().map(|c| c.name().to_string()).collect();
            }
            SimpleQueryMessage::Row(row) => {
                server_row_count += 1;
                if (rows.len() as u32) < row_limit {
                    let cells: Vec<String> = (0..row.len())
                        .map(|i| {
                            row.try_get(i)
                                .ok()
                                .flatten()
                                .map(|s: &str| s.to_string())
                                .unwrap_or_else(|| "NULL".to_string())
                        })
                        .collect();
                    rows.push(cells);
                } else {
                    truncated = true;
                }
            }
            SimpleQueryMessage::CommandComplete(_) => {}
            _ => {}
        }
    }
    Ok(QueryResult {
        columns,
        rows,
        elapsed,
        server_row_count,
        truncated,
    })
}
