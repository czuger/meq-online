# Middle-earth Quest online

An online, multiplayer version of the board game *Middle-earth Quest*: one player is
Sauron, the others play heroes. Project stalled; pre-alpha.

Written in Rust: [axum](https://github.com/tokio-rs/axum) web server,
[askama](https://github.com/askama-rs/askama) templates, SQLite through
[sqlx](https://github.com/launchbadge/sqlx). The interface is in French.

## Running

```sh
cargo run --release
```

Then open <http://127.0.0.1:3000> and log in with any name and e-mail address (developer
login, no password). Creating a game makes you Sauron; the four heroes are computer
seats that the members of the game play.

Settings (environment variables):

| Variable         | Default                 | Meaning                         |
|------------------|-------------------------|---------------------------------|
| `MEQ_BIND`       | `127.0.0.1:3000`        | Listening address               |
| `DATABASE_URL`   | `sqlite://meq.sqlite3`  | SQLite database (created if missing) |
| `MEQ_DATA_DIR`   | `data`                  | Game data (YAML)                |
| `MEQ_STATIC_DIR` | `static`                | Pictures, CSS and JavaScript    |
| `RUST_LOG`       | `info`                  | Log filter                      |

The schema (`schema.sql`) is applied at every start.

## Development

```sh
cargo test     # rules, persistence and HTTP flows
cargo clippy --all-targets
cargo fmt
```

See `CLAUDE.md` for the architecture and the coding conventions.

## Layout

- `src/` – the application (`engine/` holds the game rules).
- `templates/` – HTML pages.
- `static/` – pictures, CSS and JavaScript.
- `data/` – card, location, monster, plot and hero definitions.
- `work/` – one-off asset preparation scripts and the original rules material.
