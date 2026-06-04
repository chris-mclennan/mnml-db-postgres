# mnml-db-postgres

Postgres query playground for [mnml](https://mnml.sh) — terminal TUI
with multiple saved connections and a results table. First of the
database-viewer integration class; sibling shape to
[mnml-tickets-jira](https://github.com/chris-mclennan/mnml-tickets-jira)
/ [linear](https://github.com/chris-mclennan/mnml-tickets-linear)
/ [github](https://github.com/chris-mclennan/mnml-tickets-github),
just with a query editor instead of a static filter.

```
┌─ connections ────────────────────────────────────────────────────┐
│ ▸● Alt+1 local   ○ Alt+2 prod-api   ○ Alt+3 staging              │
└──────────────────────────────────────────────────────────────────┘
┌─ query @ local ──────────────────────────────────────────────────┐
│                                                                  │
│ SELECT id, email, created_at FROM users ORDER BY id DESC LIMIT│   │
│                                                                  │
│   Ctrl+Enter / F5 run · Ctrl+U clear · Ctrl+↑/↓ scroll results   │
└──────────────────────────────────────────────────────────────────┘
┌─ results (142 · 23ms) ───────────────────────────────────────────┐
│ id      │ email             │ created_at                          │
│ 1234567 │ alice@example.com │ 2026-06-01 14:23:11                 │
│ …                                                                │
└──────────────────────────────────────────────────────────────────┘
 142 rows · 23ms
```

## Install

```sh
cargo install --git https://github.com/chris-mclennan/mnml-db-postgres mnml-db-postgres
```

(Homebrew tap + binary releases follow once the binary stabilises.)

## Setup

1. **Run once** to scaffold the config:
   ```sh
   mnml-db-postgres
   ```
   This writes `~/.config/mnml-db-postgres.toml` and exits with
   instructions. `chmod 600` the file (it can hold DSN passwords).

2. **Edit `[[connections]]`** with your DSNs. The format is whatever
   `tokio-postgres` accepts — `postgresql://user:pass@host:port/db`
   is the canonical shape. `${ENV_VAR}` references are expanded at
   load time so you can keep secrets out of the file:

   ```toml
   [[connections]]
   name = "prod-api"
   dsn  = "postgresql://api_readonly:${PROD_DB_PASS}@db.prod.example.com:5432/api"
   ```

3. **Re-run** — the TUI launches; type a query, `Ctrl+Enter` to run.

4. **Verify** the resolved config + masked connection list:
   ```sh
   mnml-db-postgres --check
   ```
   Passwords are redacted from the printed DSN.

## Keys

| Chord                | Action                                            |
|----------------------|---------------------------------------------------|
| `Ctrl+Enter` / `F5`  | Run the current query                             |
| `Alt+1`-`Alt+9`      | Switch to that connection                         |
| `Ctrl+U`             | Clear the query buffer                            |
| `Ctrl+↑/↓` / `Ctrl+P/N` | Move selection in the results table             |
| `PgUp` / `PgDn`      | Jump 10 rows                                      |
| `Ctrl+Home` / `Ctrl+End` | Top / bottom of results                       |
| `R` (uppercase)      | Double `row_limit` for the next run               |
| `q` / `Esc` / `Ctrl+C` | Quit                                            |
| `Ctrl+D` (empty editor) | Quit                                           |

## Safety: read-only by convention

mnml-db-postgres doesn't restrict what you can type — `tokio-postgres`
runs whatever statement you give it. Use a **read-only Postgres role**
for production DSNs. The intended use is exploration / debugging,
not migrations.

## Status & roadmap

**v0.1 (this release):**
- Standalone TUI
- Multiple connections, switch via `Alt+1-9`
- Single-line query editor
- Results table with row limit + truncation marker
- Blit mode (`--blit <socket>`) so mnml/tmnl can host as a pane
- DSN password redaction in `--check`
- `${ENV_VAR}` expansion in DSNs at load time

**Planned (v0.2):**
- Multi-line query editor
- Query history (saved + recall with Ctrl+R)
- Result column widths from cell content
- Rich-type formatting (jsonb pretty-print, timestamp formatting,
  numeric alignment, NULL styling)
- Result CSV / JSON / clipboard export
- Schema browser (left pane: catalogs → schemas → tables → columns)
- Explain plan view

**Future siblings** in the same database-viewer class:
- mnml-db-mysql
- mnml-db-redis
- mnml-db-sqlite
- mnml-db-duckdb

## License

MIT.
