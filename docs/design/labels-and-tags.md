# Labels and tags

Two words, two shapes. Pirsig never uses one for the other.

| | Shape | Example | Answers | Obsidian source (BYOX) |
|---|---|---|---|---|
| **Label** | value only | `running`, `area/health/running` | "what is this about?" | `#tag`, `#nested/tag`, frontmatter `tags:` |
| **Tag** | key and value | `status=active`, `source=kindle`, `energy=low` | "what is this property set to?" | frontmatter properties (`status: active`), inline `status:: active` |

- A **label** is a bare value. An item has a set of labels; adding one twice is a no-op. A
  nested label (`area/health/running`) is still one value: the `/` only makes prefix
  queries possible (`area` matches `area/health/running`), the way Obsidian nests tags.
- A **tag** is a `key=value` pair. An item has at most one value per key (a map), so
  setting `status=done` replaces `status=active`. Keys are lowercase words
  (`[a-z][a-z0-9_-]*`); values are free text.
- Smart labels (saved queries such as `@overdue`) are labels whose membership is computed;
  they stay value-only. See the BYOL issue.

## Syntax

| | CLI | API (JSON) | Query | Markdown |
|---|---|---|---|---|
| Label | `--label running` (repeatable) | `"labels": ["running"]` | `?label=running` (prefix match) | `#running` in the body, `tags: [running]` in front matter |
| Tag | `--tag status=active` (repeatable) | `"tags": {"status": "active"}` | `?tag=status=active`, `?tag=status` (has key) | `status: active` in front matter, `status:: active` inline |

Obsidian calls value-only marks "tags". Pirsig reads and writes them in Obsidian's syntax,
but inside Pirsig they are **labels**; Obsidian's key/value properties are Pirsig **tags**. The
round trip is lossless because the shapes match one to one.

## Where Pirsig is today

- PARA (areas, projects, tasks, resources) has neither yet; the BYOL issue adds labels
  and this note adds tags next to them.
- Notes store `#hashtags` and `psg note --tag X` in `note_tags`, exposed as `tags: [..]`.
  Those are **labels** by this note's definition. Planned rename, with a deprecation
  window: `note_tags` → `note_labels`, the JSON field `tags` → `labels`, `--tag X` →
  `--label X`; `--tag` then takes `key=value`. `kind` and `source` on notes are tags in
  all but storage and stay columns.
- Exports: `psg graph` will carry labels as a node's `labels` list and tags as node
  attributes; `psg figure` can show labels in a sublabel. Neither invents a third shape.

## Out of scope

Typed tag values (numbers, dates) and multi-valued keys. If a key needs several values,
it is a set of labels (`author/luhmann`, `author/ahrens`), not a tag.
