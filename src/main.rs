mod app;
mod blit;
mod config;
mod keys;
mod postgres;
mod ui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mnml-db-postgres",
    version,
    about = "Postgres query viewer for mnml"
)]
struct Cli {
    /// Print the resolved config + connection list and exit.
    #[arg(long)]
    check: bool,
    /// Blit-host mode — render into a UDS-served cell grid instead of
    /// the local terminal. Used by mnml / tmnl to host this binary as
    /// a pane (`:host.launch mnml-db-postgres --blit /tmp/x.sock`).
    #[arg(long, value_name = "SOCKET")]
    blit: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;

    if cli.check {
        println!("config: {}", config::config_path().display());
        println!("row_limit: {}", cfg.row_limit);
        for (i, c) in cfg.connections.iter().enumerate() {
            // Redact the password if it looks like one.
            let dsn = scrub_dsn(&c.dsn);
            println!("  connection {} ({}): {}", i + 1, c.name, dsn);
        }
        return Ok(());
    }

    let mut app = app::App::new(cfg).await?;

    if let Some(socket) = cli.blit {
        blit::run(&mut app, std::path::Path::new(&socket)).await
    } else {
        ui::run(&mut app).await
    }
}

/// Redact `:<pass>@` in a Postgres DSN for terminal display. Best-
/// effort regex-free; assumes one `://user:pass@` segment.
fn scrub_dsn(dsn: &str) -> String {
    let Some(scheme_end) = dsn.find("://") else {
        return dsn.to_string();
    };
    let rest = &dsn[scheme_end + 3..];
    let Some(at) = rest.find('@') else {
        return dsn.to_string();
    };
    let userinfo = &rest[..at];
    let Some(colon) = userinfo.find(':') else {
        return dsn.to_string();
    };
    let user = &userinfo[..colon];
    let prefix = &dsn[..scheme_end + 3];
    let suffix = &rest[at..];
    format!("{prefix}{user}:****{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_dsn_hides_password() {
        let s = scrub_dsn("postgresql://api:hunter2@db.example.com:5432/api");
        assert_eq!(s, "postgresql://api:****@db.example.com:5432/api");
    }

    #[test]
    fn scrub_dsn_without_password_is_idempotent() {
        let s = scrub_dsn("postgresql://localhost:5432/postgres");
        assert_eq!(s, "postgresql://localhost:5432/postgres");
    }
}
