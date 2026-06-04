# Contributing to mnml-db-postgres

Thanks for taking a look! This repo is part of the [mnml integration family](https://mnml.sh/manual/integrations/community/) — a standalone PostgreSQL TUI that doubles as a hosted mnml pane.

## Two paths

**A. You want to fix a bug or add a Postgres-specific feature here.** Open an issue or PR against this repo. See "Local development" below.

**B. You want a viewer for a different backend** (MySQL flavor mnml doesn't have, an internal DB, a SaaS API). **Fork this repo** and replace `src/postgres.rs` with your backend. The rest of the scaffold (`blit.rs`, `config.rs`, `ui.rs`, `keys.rs`, `app.rs`) is designed to be copy-pasted. See [Building integrations](https://mnml.sh/manual/integrations/building/) for the full guide. You don't owe anything back to this repo or to mnml — your fork can live under your own name.

## Project layout

```
src/
├── main.rs                # CLI + mode dispatch (TUI / --blit / --check)
├── app.rs                 # state — connections, query buffer, results
├── config.rs              # ~/.config/mnml-db-postgres.toml
├── postgres.rs            # ← the backend-specific file (swap this when forking)
├── keys.rs                # action enum + key bindings
├── ui.rs                  # ratatui draw + crossterm loop
└── blit.rs                # tmnl-protocol over UDS — copied verbatim
```

`blit.rs` is shared verbatim across the family. Patches to `blit.rs` should land first in [`mnml-db-postgres`](https://github.com/chris-mclennan/mnml-db-postgres) and then be ported to the siblings.

## Local development

```sh
git clone https://github.com/chris-mclennan/mnml-db-postgres
cd mnml-db-postgres
cargo build
cargo test
cargo clippy --all-targets        # must be warning-free
cargo fmt                          # before committing
```

Spin up a local Postgres for manual testing:

```sh
docker run -d --name pg-mnml -p 5432:5432 -e POSTGRES_PASSWORD=dev postgres:16
cargo run -- --check                # should print "ok" for a default connection
cargo run                           # opens the TUI
```

## PR conventions

- One commit per logical change is fine; squash on merge is fine too.
- Commit messages: short imperative subject (≤72 chars), optional body explaining "why".
- Add a unit test for any backend behavior you change (`src/postgres.rs` has examples).
- `cargo clippy --all-targets` and `cargo fmt --check` must be clean.

## License + ownership

MIT. Contributions are accepted under the same license. No copyright assignment required; you keep authorship of your changes.
