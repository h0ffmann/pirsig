#!/usr/bin/env python3
"""Loads notes/*.md into a Pirsig slipbox, so the graph can be browsed with `psg box`.

    DATABASE_URL="sqlite://$PWD/slipbox.sqlite?mode=rwc" PSG=target/debug/psg \
        python3 examples/slipbox/load.py

Each file becomes one `psg note` with its kind, source and tags; its `status` becomes a
`status/<status>` tag, since Pirsig notes have no status field yet. `[[<id>-<slug>]]` links are
rewritten to `[[Title]]`, which Pirsig resolves on save (also when the target comes later).
`index.md` becomes the hub note. Only the standard library is used.
"""

import os
import pathlib
import re
import subprocess

PSG = os.environ.get("PSG", "pirsig")
NOTES = pathlib.Path(__file__).resolve().parent / "notes"
LINK = re.compile(r"\[\[([^\]|]+)\]\]")


def parse(path):
    text = path.read_text()
    _, front, body = text.split("---\n", 2)
    meta = {}
    for line in front.splitlines():
        key, _, value = line.partition(": ")
        meta[key] = value.strip().strip('"')
    meta["tags"] = [t.strip() for t in meta.get("tags", "[]").strip("[]").split(",") if t.strip()]
    # The `# Title` line repeats the title; Pirsig keeps the title apart.
    body = re.sub(r"\A\s*# .*\n", "", body).strip() + "\n"
    return meta, body


def main():
    notes = {p.stem: parse(p) for p in sorted(NOTES.glob("2*.md"))}
    titles = {stem: meta["title"] for stem, (meta, _) in notes.items()}

    def relink(body):
        return LINK.sub(lambda m: f"[[{titles[m.group(1)]}]]", body)

    for meta, body in notes.values():
        args = [PSG, "note", "-c", "-t", meta["title"], "-k", meta["type"], "-b", "-"]
        if meta["type"] == "literature":
            args += ["--source", meta["source"]]
        for tag in meta["tags"] + [f"status/{meta['status']}"]:
            args += ["--tag", tag]
        subprocess.run(args, input=relink(body), text=True, check=True, stdout=subprocess.DEVNULL)

    meta, body = parse(NOTES / "index.md")
    subprocess.run(
        [PSG, "note", "-c", "-t", meta["title"], "-k", "hub", "-b", "-"],
        input=relink(body),
        text=True,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    subprocess.run([PSG, "box", "stats"], check=True)


if __name__ == "__main__":
    main()
