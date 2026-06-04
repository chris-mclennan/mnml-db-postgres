//! Config file at `~/.config/mnml-db-postgres.toml`. First run writes
//! the scaffold + exits with instructions.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Result-row cap per query. Keeps a runaway `SELECT *` from a
    /// 10M-row table from buffering the whole thing in memory. The
    /// query editor's `r` (re-run with double limit) doubles this
    /// at runtime if you really do need more.
    #[serde(default = "default_row_limit")]
    pub row_limit: u32,
    /// Saved connections — at least one required. Switch between
    /// them via 1-9 in the TUI.
    #[serde(default)]
    pub connections: Vec<Connection>,
}

fn default_row_limit() -> u32 {
    500
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Human label shown in the connection strip (e.g. `prod-api`).
    pub name: String,
    /// Postgres connection string — `postgresql://user:pass@host:port/db`.
    /// Anything `tokio-postgres` accepts is fine. The file should be
    /// `chmod 600` since credentials live here in plaintext;
    /// `${ENV_VAR}` expansion happens at load time so you can keep
    /// passwords out of the file itself.
    pub dsn: String,
}

impl Config {
    pub const EXAMPLE: &'static str = r##"# mnml-db-postgres config. Edit and re-run.
#
# Connection strings live here in plaintext — chmod 600 the file.
# `${ENV_VAR}` references are expanded at load time, so you can keep
# the password out of the config and in your shell env / vault.

# How many rows to render per query. Doubled at runtime by pressing
# `R` if you need more.
row_limit = 500

[[connections]]
name = "local"
dsn = "postgresql://postgres@localhost:5432/postgres"

# [[connections]]
# name = "prod-api"
# dsn = "postgresql://api_readonly:${PROD_DB_PASS}@db.prod.example.com:5432/api"

# [[connections]]
# name = "staging"
# dsn = "postgresql://api_readonly:${STAGING_DB_PASS}@db.staging.example.com:5432/api"
"##;

    pub fn validate(&self) -> Result<()> {
        if self.connections.is_empty() {
            return Err(anyhow!(
                "config: at least one [[connections]] entry required"
            ));
        }
        if self.row_limit == 0 {
            return Err(anyhow!("config: row_limit must be > 0"));
        }
        for (i, c) in self.connections.iter().enumerate() {
            if c.name.trim().is_empty() {
                return Err(anyhow!("connection #{i}: `name` is required"));
            }
            if c.dsn.trim().is_empty() {
                return Err(anyhow!("connection #{i} ({}): `dsn` is required", c.name));
            }
        }
        Ok(())
    }

    /// Expand `${ENV_VAR}` references in each connection's DSN. Run
    /// once at load time so the TUI doesn't have to re-resolve.
    /// Missing env vars leave the literal `${NAME}` in place (so the
    /// connection attempt fails with a clear message rather than
    /// silently using an empty string for the password).
    pub fn expand_env(&mut self) {
        for c in self.connections.iter_mut() {
            c.dsn = expand_env(&c.dsn);
        }
    }
}

fn expand_env(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            match std::env::var(&name) {
                Ok(v) => out.push_str(&v),
                Err(_) => {
                    // Leave the literal — caller sees a clear error
                    // on connect rather than a silent empty value.
                    out.push_str("${");
                    out.push_str(&name);
                    out.push('}');
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("mnml-db-postgres.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, Config::EXAMPLE)?;
        return Err(anyhow!(
            "wrote config template to {} — edit it (chmod 600!) then re-run",
            path.display()
        ));
    }
    let text = std::fs::read_to_string(&path)?;
    let mut cfg: Config = toml::from_str(&text)?;
    cfg.validate()?;
    cfg.expand_env();
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_parses_and_validates() {
        let cfg: Config = toml::from_str(Config::EXAMPLE).unwrap();
        cfg.validate().unwrap();
        assert!(!cfg.connections.is_empty());
        assert_eq!(cfg.row_limit, 500);
    }

    #[test]
    fn validate_rejects_empty_connections() {
        let cfg: Config = toml::from_str("row_limit = 100").unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_zero_row_limit() {
        let raw = r##"
row_limit = 0
[[connections]]
name = "x"
dsn = "postgresql://localhost/db"
"##;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn expand_env_substitutes_known_vars() {
        // SAFETY: tests are single-threaded at env-mutation; localized var.
        unsafe { std::env::set_var("MNML_DB_PG_TEST", "hunter2") };
        let s = expand_env("postgresql://u:${MNML_DB_PG_TEST}@h/d");
        assert_eq!(s, "postgresql://u:hunter2@h/d");
        unsafe { std::env::remove_var("MNML_DB_PG_TEST") };
    }

    #[test]
    fn expand_env_leaves_unknown_vars_in_place() {
        let s = expand_env("postgresql://u:${DEFINITELY_NOT_SET_zzz}@h/d");
        assert!(s.contains("${DEFINITELY_NOT_SET_zzz}"));
    }
}
