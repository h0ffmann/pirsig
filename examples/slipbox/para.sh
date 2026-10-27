#!/usr/bin/env sh
# Seeds a ztk database with the PARA map of docs/use-cases.md: the four lights as areas, the open
# outcomes as projects, the public repositories and forks as resources, the archived ones in the
# archive. Only public repositories are named.
#
#   DATABASE_URL="sqlite://$PWD/slipbox.sqlite?mode=rwc" examples/slipbox/para.sh
#
# ZTK picks the binary (default: `ztk` on the PATH, e.g. ZTK=target/debug/ztk).
set -eu

ZTK=${ZTK:-ztk}
GH=https://github.com

# `ztk <kind> -c ...` prints "#<id> <name>"; keep the id.
id() { "$ZTK" "$@" | sed -n 's/^#\([0-9]*\).*/\1/p'; }

# Areas: one per light (h0ffmann/h0ffmann, DESIGN.md).
water=$(id area -c -n Water --description "blue: the sea, and what is measured or built for it")
ideas=$(id area -c -n Ideas --description "green: knowledge worked out and written down")
ouro=$(id area -c -n Ouroboric --description "orange: open factories that improve themselves")
closed=$(id area -c -n "Closed source" --description "red: closed source, and the case against it")

# Projects: outcomes with an end.
bench=$(id project -c -n "marola-bench v1: forecasts scored against buoys" -a "$water" --status active)
ww3=$(id project -c -n "ww3-gpu: all of model/src bit for bit on a GPU" -a "$water" --status active)
gpt=$(id project -c -n "marola-gpt 0.1.0 on a reviewed corpus" -a "$water" --status active)
writer=$(id project -c -n "ptbr-writer browser extension" -a "$ideas" --status active)
ztk=$(id project -c -n "ztk v0" -a "$ouro" --status active)
book=$(id project -c -n "closed-source v1" -a "$closed" --status active)

# Tasks: next actions the READMEs name.
"$ZTK" task -c -t "Review every marola-corpus note sentence by sentence" -p "$gpt"
"$ZTK" task -c -t "Get the book peer reviewed" -p "$book"
"$ZTK" task -c -t "Import my repositories as literature notes" -p "$ztk"
"$ZTK" task -c -t "Approve or reject the partial notes of examples/slipbox" -p "$ztk"

# Resources: the public repositories, in their area and, when one fits, their project.
res() { title=$1 url=$2 area=$3; shift 3; "$ZTK" resource -c -t "$title" --url "$url" -a "$area" "$@"; }

res "marola" "$GH/marola-dev/marola" "$water"
res "marola-research" "$GH/marola-dev/marola-research" "$water" -p "$bench"
res "ww3-gpu (site; the repository is private)" "https://h0ffmann.github.io/ww3-gpu/" "$water" -p "$ww3"
res "WW3 (fork of NOAA-EMC/WW3)" "$GH/h0ffmann/WW3" "$water" -p "$ww3"
res "WW4 (fork of NOAA-EMC/WW4)" "$GH/h0ffmann/WW4" "$water"
res "wavespectra (fork)" "$GH/h0ffmann/wavespectra" "$water"
res "marola-corpus" "$GH/marola-dev/marola-corpus" "$water" -p "$gpt"
res "marola-ml" "$GH/marola-dev/marola-ml" "$water" -p "$gpt"
res "marola-skills" "$GH/marola-dev/marola-skills" "$water"
res "awesome-ocean-science" "$GH/h0ffmann/awesome-ocean-science" "$water"

res "picasso" "$GH/h0ffmann/picasso" "$ideas"
res "nashkell" "$GH/h0ffmann/nashkell" "$ideas"
res "ptbr-writer" "$GH/h0ffmann/ptbr-writer" "$ideas" -p "$writer"
res "forecast-energy-demand" "$GH/h0ffmann/forecast-energy-demand" "$ideas"
res "gcp-agentic-architect" "$GH/h0ffmann/gcp-agentic-architect" "$ideas"
res "h0ffmann (the profile, and its slip box)" "$GH/h0ffmann/h0ffmann" "$ideas"
res "scivolo (fork)" "$GH/h0ffmann/scivolo" "$ideas"

res "ouroboric" "$GH/ouroboric-dev/ouroboric" "$ouro"
res "zettelkasten (this fork)" "$GH/h0ffmann/zettelkasten" "$ouro" -p "$ztk"
res "loco (fork)" "$GH/h0ffmann/loco" "$ouro" -p "$ztk"
res "basic-memory (fork)" "$GH/h0ffmann/basic-memory" "$ouro" -p "$ztk"
res "ai-memory (fork)" "$GH/h0ffmann/ai-memory" "$ouro" -p "$ztk"
res "onyx (fork)" "$GH/h0ffmann/onyx" "$ouro" -p "$ztk"

res "closed-source" "$GH/h0ffmann/closed-source" "$closed" -p "$book"

# Archives: repositories GitHub marks archived (or that say so in their README).
for r in smart-notes montaigne mdd-risk-analysis; do
  rid=$(res "$r (archived)" "$GH/h0ffmann/$r" "$ideas" | sed -n 's/^#\([0-9]*\).*/\1/p')
  "$ZTK" resource --archive "$rid"
done

"$ZTK" para
