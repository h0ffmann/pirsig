# ztk tasks. `just` lists them; `just <name> args...` runs one.
# Rust does the work, just only remembers the commands.

set dotenv-load := true

ztk := "cargo run -q --bin ztk --"

default:
    @just --list --unsorted

# --- daily use -------------------------------------------------------------

# Capture a fleeting note: just n "An idea" [-b "more"] [--tag x]
n title *args:
    {{ztk}} note -c "{{title}}" {{args}}

# Continue a train of thought: just after 21 "Next idea"
after zid title *args:
    {{ztk}} note -c "{{title}}" -a {{zid}} {{args}}

# Show a note with links and backlinks
show zid:
    {{ztk}} note -s {{zid}}

# Edit a note's body in $EDITOR
edit zid:
    {{ztk}} note -u {{zid}} -e

# Search titles and bodies
find query:
    {{ztk}} note -q "{{query}}"

# Link two notes with a reason: just link 21a 3 "contradicts"
link from to why="":
    {{ztk}} link {{from}} {{to}} -w "{{why}}"

# Slipbox views: stats | inbox | orphans | random | tree | tags | graph
box view="stats":
    {{ztk}} box {{view}}

# Export every note as Markdown into a folder (default ./vault)
export dir="vault":
    {{ztk}} box export {{dir}}

# Run any ztk command: just ztk para --json
ztk *args:
    {{ztk}} {{args}}

# Install ztk on your PATH
install:
    cargo install --path . --bin ztk --locked

# --- development -----------------------------------------------------------

# API server on http://localhost:5150
dev:
    cargo loco start

# Format, lint and test: what CI runs
check: fmt-check lint test

test *args:
    cargo test --all-features --all {{args}}

lint:
    cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Apply pending migrations and regenerate src/models/_entities
migrate:
    cargo loco db migrate
    cargo loco db entities

# New model with migration: just model notes title:string!
model name *fields:
    cargo loco generate model {{name}} {{fields}}

# Drop the development database and start over
reset:
    cargo loco db reset
