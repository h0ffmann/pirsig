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

# Methodology readiness: PARA, Zettelkasten, Obsidian, RSS, arXiv
ready:
    {{ztk}} ready

# Seed the brain from your GitHub repos: just bootstrap --private minimal --readme
bootstrap *args:
    GH_TOKEN=$(gh auth token) {{ztk}} bootstrap gh {{args}}

# A quick Plasticity session: five exercises
plast:
    {{ztk}} plast go

# Build the obsidian-painter plugin (Rust core to WebAssembly, then main.js) and test it
painter:
    cd obsidian-painter && npm ci && npm run build && npm run check && npm test

# Open the desktop app (Tauri): the whole brain, on the same database as `ztk`
desktop:
    cargo run --manifest-path desktop/src-tauri/Cargo.toml

# The UI end to end in WebKit, on a fresh throwaway brain (Playwright)
desktop-e2e:
    cd desktop && npm ci && npx playwright install webkit && npx playwright test

# Format, lint and test: what CI runs
check: fmt-check lint test

# Alias for `check`: the marola-devkit stop hook asks for `just quality`
quality: check

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

# --- figures and graphs ----------------------------------------------------

# Check out the pinned Archify (vendor/archify, sparse to archify/), once per clone
figure-init:
    git submodule update --init --depth 1 vendor/archify
    git -C vendor/archify sparse-checkout set --no-cone /archify/
    @echo "figure-init: $(git -C vendor/archify rev-parse --short HEAD)"

# Archify validates and checks each source (finalize), picasso's look (docs/archify/picasso.css) goes
# over it, headless Chrome takes the picture. Needs node, jq, perl, Chrome (CHROME names another) and
# the Google Fonts; writes docs/archify/figures.lock.
# Draw every docs/archify/<slug>.json to docs/img/<slug>.png
figure: figure-init
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p build/figure docs/img
    for src in docs/archify/*.json; do
        slug="$(basename "$src" .json)"; out="build/figure/$slug"
        rm -rf "$out"; mkdir -p "$out"
        ARCHIFY_UPDATE_CHECK_DISABLED=1 node vendor/archify/archify/bin/archify.mjs finalize "$(jq -r .diagram_type "$src")" "$src" "$out/page.html" --out-dir "$out/receipts" --quality "$(jq -r '.meta.quality_profile // "showcase"' "$src")"
        read -r w h < <(grep -o '<svg viewBox="[^"]*"' "$out/page.html" | head -1 | sed -E 's/.*0 0 ([0-9.]+) ([0-9.]+)"/\1 \2/')
        crop="$(awk -F'\t' -v s="$slug" '$1==s {print $2}' docs/archify/crop.tsv)"; h="${crop:-$h}"
        FONTS='<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Fjalla+One&family=Inter:wght@400;600&display=block">' CSS="$(cat docs/archify/picasso.css)" W="$w" H="$h" \
            perl -0pe 's|</head>|$ENV{FONTS}<style>$ENV{CSS}</style></head>|; s|<svg viewBox="0 0 [0-9.]+ [0-9.]+"|<svg viewBox="0 0 $ENV{W} $ENV{H}"|' "$out/page.html" > "$out/frame.html"
        "${CHROME:-google-chrome}" --headless --disable-gpu --hide-scrollbars --force-dark-mode --force-device-scale-factor=2 --user-data-dir="$PWD/$out/profile" --window-size="$w,$h" --virtual-time-budget=3000 --screenshot="$PWD/docs/img/$slug.png" "file://$PWD/$out/frame.html" 2>/dev/null
        echo "figure: docs/img/$slug.png"
    done
    just _figure-lock > docs/archify/figures.lock

# The demo brain lives in a throwaway SQLite file, so the PARA picture is the CLI's own output;
# `just figure` then draws it.
# Write docs/archify/para.json with `ztk figure` from a demo brain
figure-para:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p build/figure
    db="$PWD/build/figure/demo.sqlite"; rm -f "$db"
    export DATABASE_URL="sqlite://$db?mode=rwc"
    cargo build -q --bin ztk
    z=target/debug/ztk
    $z area -c -n Health --description "body and mind" >/dev/null
    $z area -c -n Work >/dev/null
    $z area -c -n "Second brain" >/dev/null
    $z project -c -n "Run a marathon" -a 1 --status active >/dev/null
    $z project -c -n "Q4 roadmap" -a 2 --status planning >/dev/null
    $z project -c -n "Ship ztk v0" -a 3 --status active >/dev/null
    $z task -c -t "Buy shoes" -p 1 --due 2026-11-01 >/dev/null
    $z task -c -t "Long run, 21 km" -p 1 >/dev/null
    $z task -c -t "Obsidian sync" -p 3 >/dev/null
    $z task -c -t "Archify figures" -p 3 >/dev/null
    $z resource -c -t "Couch to 5k" --url https://example.com/c25k -p 1 >/dev/null
    $z resource -c -t "PARA method" -a 3 >/dev/null
    $z resource -c -t "Reading list" >/dev/null
    $z figure -t "A second brain in ztk" -o docs/archify/para.json

# The inputs of each figure (its source, the look, the crop, the Archify pin), one line per slug.
_figure-lock:
    #!/usr/bin/env bash
    set -euo pipefail
    pin="$(git ls-files -s vendor/archify | cut -d' ' -f2)"
    for src in docs/archify/*.json; do
        printf '%s\t%s\n' "$(basename "$src" .json)" "$(cat "$src" docs/archify/picasso.css docs/archify/crop.tsv | sha256sum | cut -c1-16)-${pin:0:7}"
    done

# Cheap and offline: every source has its PNG, README.md shows it, both match the last `just figure`
figure-check:
    #!/usr/bin/env bash
    set -euo pipefail
    for src in docs/archify/*.json; do
        s="$(basename "$src" .json)"
        test -s "docs/img/$s.png" || { echo "figure-check: no PNG for $src; run just figure" >&2; exit 1; }
        grep -qF "](docs/img/$s.png)" README.md || { echo "figure-check: README.md does not show docs/img/$s.png" >&2; exit 1; }
    done
    just _figure-lock | cmp -s - docs/archify/figures.lock || { echo "figure-check: a source, the look or the Archify pin changed since the PNG; run just figure" >&2; exit 1; }
    echo "figure-check: ok"

# Writes graphify-out/graph.json, then communities, GRAPH_REPORT.md and graph.html. Needs graphify
# on PATH (pip install graphifyy==0.9.96); DATABASE_URL picks the brain.
# The brain as a graphify knowledge graph
graph:
    {{ztk}} graph -o graphify-out/graph.json
    graphify cluster-only . --no-label
    @echo "graph: graphify-out/graph.html; ask with: graphify query \"...\""
