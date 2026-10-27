//! Keeps `examples/slipbox` honest: the rules of the owner's slip box (h0ffmann/h0ffmann, NOTES.md).

use std::{collections::BTreeMap, fs, path::Path};

use regex::Regex;

const NOTES: &str = "examples/slipbox/notes";
const TYPES: [&str; 3] = ["fleeting", "literature", "permanent"];
const STATUSES: [&str; 5] = [
    "unverified",
    "partial",
    "verified",
    "open-question",
    "approved",
];

struct Note {
    front: BTreeMap<String, String>,
    links: Vec<String>,
}

fn read_notes() -> BTreeMap<String, Note> {
    let link = Regex::new(r"\[\[([^\]|]+)\]\]").unwrap();
    let mut notes = BTreeMap::new();
    for entry in fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(NOTES)).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if name == "index" {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let (front, _) = text
            .strip_prefix("---\n")
            .and_then(|t| t.split_once("\n---\n"))
            .unwrap_or_else(|| panic!("{name}: no front matter"));
        let front = front
            .lines()
            .filter_map(|l| l.split_once(": "))
            .map(|(k, v)| (k.to_string(), v.trim_matches('"').to_string()))
            .collect();
        let links = link
            .captures_iter(&text)
            .map(|c| c[1].to_string())
            .collect();
        notes.insert(name, Note { front, links });
    }
    notes
}

#[test]
fn every_note_follows_the_slip_box_rules() {
    let notes = read_notes();
    assert!(!notes.is_empty());
    for (name, note) in &notes {
        let id = &note.front["id"];
        assert!(
            id.len() == 12 && id.bytes().all(|b| b.is_ascii_digit()),
            "{name}: id {id}"
        );
        assert!(
            name.starts_with(&format!("{id}-")),
            "{name}: file is not <id>-<slug>"
        );
        for key in ["title", "source", "tags", "created"] {
            assert!(note.front.contains_key(key), "{name}: no {key}");
        }
        assert!(TYPES.contains(&note.front["type"].as_str()), "{name}: type");
        assert!(
            STATUSES.contains(&note.front["status"].as_str()),
            "{name}: status"
        );
        assert!(!note.links.is_empty(), "{name}: links to no other note");
        for target in &note.links {
            assert!(notes.contains_key(target), "{name}: dead link [[{target}]]");
            assert_ne!(target, name, "{name}: links to itself");
        }
        if note.front["type"] == "permanent" {
            assert!(note.links.len() >= 2, "{name}: a tie needs two notes");
        }
    }
}

#[test]
fn the_index_links_every_note() {
    let index = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(NOTES)
            .join("index.md"),
    )
    .unwrap();
    for name in read_notes().keys() {
        assert!(
            index.contains(&format!("[[{name}]]")),
            "index misses {name}"
        );
    }
}
