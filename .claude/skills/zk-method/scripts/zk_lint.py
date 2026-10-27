#!/usr/bin/env python3
"""Mechanical Zettelkasten lint for a folder of Markdown notes.

Finds what can be counted: kinds, IDs, orphans, broken links, bare link lists,
unsourced literature notes, long permanent notes, stale fleeting notes, and
permanent notes filed in PARA folders. Atomicity and own words need a reader.

    python3 zk_lint.py <vault> [--inbox Inbox] [--stale-days 14] [--json]

Standard library only.
"""

import argparse
import datetime as dt
import json
import re
import sys
from pathlib import Path

KINDS = ("fleeting", "literature", "permanent", "structure")
KIND_ALIASES = {
    "fleeting": "fleeting", "inbox": "fleeting", "capture": "fleeting",
    "literature": "literature", "lit": "literature", "source": "literature", "reference": "literature",
    "permanent": "permanent", "zettel": "permanent", "evergreen": "permanent",
    "structure": "structure", "moc": "structure", "hub": "structure", "index": "structure",
}
FOLDER_KINDS = {
    "inbox": "fleeting", "fleeting": "fleeting",
    "literature": "literature", "lit": "literature", "sources": "literature", "references": "literature",
    "zettelkasten": "permanent", "slipbox": "permanent", "slip-box": "permanent", "permanent": "permanent",
    "zettels": "permanent", "notes": "permanent",
    "moc": "structure", "mocs": "structure", "hubs": "structure", "structure": "structure",
}
PARA_FOLDERS = {"projects", "areas", "resources", "archive", "archives"}
SKIP_DIRS = {".obsidian", ".git", ".trash", "templates", "_templates", "node_modules"}

ID_RE = re.compile(r"^(\d{12,14})")
WIKILINK_RE = re.compile(r"\[\[([^\]|#]+)(?:#[^\]|]*)?(?:\|[^\]]*)?\]\]")
MDLINK_RE = re.compile(r"\[[^\]]*\]\(([^)\s]+?\.md)(?:#[^)]*)?\)")
CITE_RE = re.compile(r"(?<![\w.])@[A-Za-z][\w:-]+\d{2,4}\w*")
URL_RE = re.compile(r"https?://\S+")
FENCE_RE = re.compile(r"```.*?```", re.S)


def parse_frontmatter(text):
    """A small YAML subset: `key: value`, `key: [a, b]` and `key:` followed by `- item` lines."""
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---", 4)
    if end < 0:
        return {}, text
    meta, key = {}, None
    for line in text[4:end].splitlines():
        item = re.match(r"^\s+-\s+(.*)$", line)
        if item and key:
            meta.setdefault(key, [])
            if isinstance(meta[key], list):
                meta[key].append(item.group(1).strip().strip("'\""))
            continue
        kv = re.match(r"^([\w-]+):\s*(.*)$", line)
        if not kv:
            continue
        key, value = kv.group(1).lower(), kv.group(2).strip()
        if value.startswith("[") and value.endswith("]"):
            meta[key] = [v.strip().strip("'\"") for v in value[1:-1].split(",") if v.strip()]
        elif value:
            meta[key] = value.strip("'\"")
        else:
            meta[key] = []
    body = text[end + 4:]
    return meta, body.split("\n", 1)[1] if "\n" in body else ""


def as_list(value):
    if value is None:
        return []
    return value if isinstance(value, list) else [value]


def link_targets(body):
    text = FENCE_RE.sub("", body)
    targets = [m.strip() for m in WIKILINK_RE.findall(text)]
    targets += [Path(m).stem for m in MDLINK_RE.findall(text) if "://" not in m]
    return targets


def is_bare_link_line(line):
    """A line that is links and little else: `- [[a]]`, `[[a]] [[b]]`, `Related: [[a]]`."""
    if not WIKILINK_RE.search(line) and not MDLINK_RE.search(line):
        return False
    rest = MDLINK_RE.sub("", WIKILINK_RE.sub("", line))
    rest = re.sub(r"^\s*([-*+]|\d+\.)\s*", "", rest)
    rest = re.sub(r"(?i)\b(related|see also|links?|refs?)\s*:?", "", rest)
    return len(re.findall(r"\w+", rest)) <= 2


def note_date(meta, note_id, path):
    for key in ("created", "date"):
        value = meta.get(key)
        if isinstance(value, str):
            try:
                return dt.date.fromisoformat(value[:10])
            except ValueError:
                pass
    if note_id and len(note_id) >= 8:
        try:
            return dt.datetime.strptime(note_id[:8], "%Y%m%d").date()
        except ValueError:
            pass
    return dt.date.fromtimestamp(path.stat().st_mtime)


def classify(meta, rel, body, out_links, words):
    for value in as_list(meta.get("type")) + as_list(meta.get("kind")):
        if value.lower() in KIND_ALIASES:
            return KIND_ALIASES[value.lower()], "frontmatter"
    for tag in as_list(meta.get("tags")):
        leaf = tag.lstrip("#").lower().split("/")[-1]
        if leaf in KIND_ALIASES:
            return KIND_ALIASES[leaf], "tag"
    for part in reversed(rel.parts[:-1]):
        name = re.sub(r"^[\d\s._-]+", "", part).lower()
        if name in FOLDER_KINDS:
            return FOLDER_KINDS[name], "folder"
    if len(out_links) >= 5 and words / max(len(out_links), 1) < 15:
        return "structure", "guessed"
    if has_source(meta, body):
        return "literature", "guessed"
    return "permanent", "guessed"


def has_source(meta, body):
    if any(meta.get(k) for k in ("source", "citekey", "url", "doi", "isbn", "author", "authors")):
        return True
    return bool(CITE_RE.search(body) or URL_RE.search(body))


def in_para_folder(rel):
    return any(re.sub(r"^[\d\s._-]+", "", p).lower() in PARA_FOLDERS for p in rel.parts[:-1])


def load(vault):
    notes = []
    for path in sorted(vault.rglob("*.md")):
        rel = path.relative_to(vault)
        if any(p.lower() in SKIP_DIRS for p in rel.parts[:-1]):
            continue
        meta, body = parse_frontmatter(path.read_text(encoding="utf-8", errors="replace"))
        id_match = ID_RE.match(path.stem)
        note_id = str(meta.get("id") or (id_match.group(1) if id_match else "")) or None
        words = len(re.findall(r"\w+", FENCE_RE.sub("", body)))
        out_links = link_targets(body)
        kind, kind_from = classify(meta, rel, body, out_links, words)
        h1 = re.findall(r"(?m)^# ", body)
        title = meta.get("title") or (re.search(r"(?m)^# (.+)$", body) or [None, path.stem])[1]
        notes.append({
            "path": str(rel), "stem": path.stem, "id": note_id, "title": title,
            "aliases": as_list(meta.get("aliases")), "kind": kind, "kind_from": kind_from,
            "words": words, "out": out_links, "h1": len(h1),
            "h2": len(re.findall(r"(?m)^## ", body)),
            "bare_lines": sum(is_bare_link_line(l) for l in body.splitlines()),
            "sourced": has_source(meta, body), "date": note_date(meta, note_id, path),
            "para": in_para_folder(rel), "findings": [],
        })
    return notes


def lint(notes, stale_days, today):
    index = {}
    for n in notes:
        for key in [n["stem"], n["id"], n["title"], *n["aliases"]]:
            if key:
                index.setdefault(str(key).lower(), n["path"])
        if n["id"]:
            index.setdefault(f'{n["id"]} {n["title"]}'.lower(), n["path"])
    inbound = {n["path"]: 0 for n in notes}
    for n in notes:
        resolved, broken = set(), []
        for t in n["out"]:
            hit = index.get(t.lower()) or index.get(t.split("/")[-1].lower())
            if hit:
                if hit != n["path"]:
                    resolved.add(hit)
            else:
                broken.append(t)
        n["resolved"] = sorted(resolved)
        for hit in resolved:
            inbound[hit] += 1
        if broken:
            n["findings"].append(("links", f"broken link(s): {', '.join(sorted(set(broken)))}"))

    for n in notes:
        n["inbound"] = inbound[n["path"]]
        add = lambda rule, msg: n["findings"].append((rule, msg))
        kind = n["kind"]
        if kind in ("permanent", "structure"):
            if not n["id"]:
                add("identity", "no stable ID (frontmatter `id:` or timestamp filename prefix)")
            if not n["resolved"] and not n["inbound"]:
                add("orphan", "no links in or out: this note cannot be reached by thinking")
            elif not n["resolved"]:
                add("links", "no outgoing links: say which older note this one answers")
        if kind == "permanent":
            if n["bare_lines"] >= 2:
                add("links", f"{n['bare_lines']} bare link lines: give each link a reason in a sentence")
            if n["words"] > 700:
                add("atomic", f"{n['words']} words: likely more than one idea, or an undistilled summary")
            if n["h2"] > 1:
                add("atomic", f"{n['h2']} `##` sections: each may be its own note")
            if n["h1"] > 1:
                add("atomic", f"{n['h1']} `#` titles in one note")
            if n["para"]:
                add("para", "permanent note inside a PARA folder: link it from the project instead")
        if kind == "literature" and not n["sourced"]:
            add("source", "literature note without a source (citekey, URL, DOI or `source:`)")
        if kind == "structure" and n["bare_lines"] > 3:
            add("links", "structure note is a bare list: one line of reason per link makes it an entry point")
        if kind == "fleeting":
            age = (today - n["date"]).days
            if age > stale_days:
                add("inbox", f"fleeting for {age} days: process into a permanent note or delete")
        if n["kind_from"] == "guessed" and n["findings"]:
            add("kind", f"kind guessed as {kind}: set `type:` in frontmatter to make it explicit")
    return notes


def summary(notes, stale_days, today):
    by_kind = {k: sum(n["kind"] == k for n in notes) for k in KINDS}
    fleeting = [n for n in notes if n["kind"] == "fleeting"]
    return {
        "notes": len(notes),
        "kinds": by_kind,
        "inbox": len(fleeting),
        "inbox_stale": sum((today - n["date"]).days > stale_days for n in fleeting),
        "orphans": sum(any(r == "orphan" for r, _ in n["findings"]) for n in notes),
        "notes_with_findings": sum(bool(n["findings"]) for n in notes),
        "links": sum(len(n["resolved"]) for n in notes),
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("vault", type=Path)
    ap.add_argument("--inbox", action="append", default=[],
                    help="extra folder name(s) that hold fleeting notes")
    ap.add_argument("--stale-days", type=int, default=14)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    if not args.vault.is_dir():
        sys.exit(f"not a folder: {args.vault}")
    for name in args.inbox:
        FOLDER_KINDS[name.lower()] = "fleeting"

    today = dt.date.today()
    notes = lint(load(args.vault), args.stale_days, today)
    report = summary(notes, args.stale_days, today)

    if args.json:
        out = [{k: (str(v) if k == "date" else v) for k, v in n.items()} for n in notes]
        for n in out:
            n["findings"] = [{"rule": r, "message": m} for r, m in n["findings"]]
        json.dump({"summary": report, "notes": out}, sys.stdout, indent=2)
        print()
        return

    rank = {"orphan": 0, "atomic": 1, "links": 2, "source": 3, "para": 4, "identity": 5, "inbox": 6, "kind": 7}
    flagged = sorted((n for n in notes if n["findings"]),
                     key=lambda n: (min(rank.get(r, 9) for r, _ in n["findings"]), n["path"]))
    for n in flagged:
        tag = n["kind"] + ("?" if n["kind_from"] == "guessed" else "")
        print(f'{n["path"]}  ({tag}, {n["words"]} words, {len(n["resolved"])} out / {n["inbound"]} in)')
        for rule, msg in n["findings"]:
            print(f"  ✗ {rule}: {msg}")
    if flagged:
        print()
    kinds = ", ".join(f"{v} {k}" for k, v in report["kinds"].items())
    print(f'{report["notes"]} notes ({kinds}); {report["links"]} resolved links')
    print(f'inbox: {report["inbox"]} fleeting, {report["inbox_stale"]} older than {args.stale_days} days')
    print(f'orphans: {report["orphans"]}; notes with findings: {report["notes_with_findings"]}')
    print("? = kind guessed. This lint counts; read the flagged notes to judge atomicity and own words.")


if __name__ == "__main__":
    main()
