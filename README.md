# Juniper

Example using Rust Actix-web (web framework) with juniper (Graphql) with diesel (ORM)

# Diesel
Diesel's `Getting Started` guide using SQLite for Actix web

## Usage

### init database sqlite

```bash
cargo install diesel_cli --no-default-features --features sqlite
cd examples/diesel
echo "DATABASE_URL=file:test.db" > .env
diesel migration run
```

### server

```bash
# if ubuntu : sudo apt-get install libsqlite3-dev
# if fedora : sudo dnf install libsqlite3x-devel
cd examples/diesel
cargo run (or ``cargo watch -x run``)
# Started http server: 127.0.0.1:8080
```

### Graphql client
```
http://localhost:8080/graphiql
```

### sqlite client

```bash
# if ubuntu : sudo apt-get install sqlite3
# if fedora : sudo dnf install sqlite3x
sqlite3 test.db
sqlite> .tables
sqlite> select * from users;
```