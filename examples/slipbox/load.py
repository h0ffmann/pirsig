#!/usr/bin/env python3
"""Loads notes/*.md into a ztk slipbox, so the graph can be browsed with `ztk box`.

    DATABASE_URL="sqlite://$PWD/slipbox.sqlite?mode=rwc" ZTK=target/debug/ztk \
        python3 examples/slipbox/load.py

Each file becomes one `ztk note` with its kind, source and tags; its `status` becomes a
`status/<status>` tag, since ztk notes have no status field yet. `[[<id>-<slug>]]` links are
rewritten to `[[Title]]`, which ztk resolves on save (also when the target comes later).
`index.md` becomes the hub note. Only the standard library is used.
"""

import os
import pathlib
import re
import subprocess

ZTK = os.environ.get("ZTK", "ztk")
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
    # The `# Title` line repeats the title; ztk keeps the title apart.
    body = re.sub(r"\A\s*# .*\n", "", body).strip() + "\n"
    return meta, body


def main():
    notes = {p.stem: parse(p) for p in sorted(NOTES.glob("2*.md"))}
    titles = {stem: meta["title"] for stem, (meta, _) in notes.items()}

    def relink(body):
        return LINK.sub(lambda m: f"[[{titles[m.group(1)]}]]", body)

    for meta, body in notes.values():
        args = [ZTK, "note", "-c", "-t", meta["title"], "-k", meta["type"], "-b", "-"]
        if meta["type"] == "literature":
            args += ["--source", meta["source"]]
        for tag in meta["tags"] + [f"status/{meta['status']}"]:
            args += ["--tag", tag]
        subprocess.run(args, input=relink(body), text=True, check=True, stdout=subprocess.DEVNULL)

    meta, body = parse(NOTES / "index.md")
    subprocess.run(
        [ZTK, "note", "-c", "-t", meta["title"], "-k", "hub", "-b", "-"],
        input=relink(body),
        text=True,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    subprocess.run([ZTK, "box", "stats"], check=True)


if __name__ == "__main__":
    main()
