use std::{
    fmt::Write,
    io::{IsTerminal, Read},
    process::Command,
};

use clap::ArgGroup;
use loco_rs::{app::AppContext, model::ModelError, Error, Result};
use sea_orm::QuerySelect;

use super::{explain, Output};
use crate::{
    dtos::notes::{CreateNote, LinkDto, NoteDetail, NoteDto, UpdateNote},
    models::notes,
};

/// `ztk note -c "An idea"`, `ztk note -c -t "Title" -a 21 -k permanent -e`,
/// `ztk note -s 21a`, `ztk note -u 21a -e`, `ztk note -d 21a`, `ztk note -q rust` (search).
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show"])))]
pub struct Args {
    /// Create a note (title as --title or as the first argument).
    #[arg(short, long)]
    create: bool,
    /// Update the note with this zid.
    #[arg(short, long, value_name = "ZID")]
    update: Option<String>,
    /// Delete the note with this zid (and its links).
    #[arg(short, long, value_name = "ZID")]
    delete: Option<String>,
    /// Show one note with its links and backlinks.
    #[arg(short, long, value_name = "ZID")]
    show: Option<String>,

    /// Note title.
    #[arg(short, long)]
    title: Option<String>,
    /// Title, for quick capture: `ztk note -c "An idea"`.
    #[arg(value_name = "TITLE")]
    quick_title: Option<String>,
    /// Body in Markdown; `-` reads it from stdin. `[[zid]]` or `[[Title]]` links, `#tag` tags.
    #[arg(short, long)]
    body: Option<String>,
    /// Write the body in $EDITOR.
    #[arg(short, long)]
    edit: bool,
    /// fleeting (default), literature, permanent or hub.
    #[arg(short, long)]
    kind: Option<String>,
    /// Where a literature note comes from (book, URL, @citekey); `""` clears it.
    #[arg(long)]
    source: Option<String>,
    /// The resource a literature note was written from (its citation becomes the source).
    #[arg(short, long, value_name = "RESOURCE_ID")]
    resource: Option<i64>,
    /// File the new note as a continuation of this one (`21` → `21a`).
    #[arg(short, long, value_name = "ZID")]
    after: Option<String>,
    /// Tag, repeatable. On update, replaces the explicit tags.
    #[arg(long = "tag", value_name = "TAG")]
    tags: Vec<String>,

    /// When listing, match this text in title or body.
    #[arg(short, long)]
    query: Option<String>,
    /// With --show, print the note as Markdown instead.
    #[arg(long)]
    md: bool,
    /// When listing, how many notes to show.
    #[arg(short = 'n', long, default_value_t = 20)]
    limit: u64,
}

pub(crate) fn line(n: &NoteDto) -> String {
    let tags = if n.tags.is_empty() {
        String::new()
    } else {
        format!(
            "  {}",
            n.tags
                .iter()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
    format!("{:<8} {:<10} {}{tags}", n.zid, n.kind, n.title)
}

fn link_line(arrow: &str, l: &LinkDto) -> String {
    let why = l
        .context
        .as_deref()
        .map(|c| format!(": {c}"))
        .unwrap_or_default();
    format!("  {arrow} {} {}{why}", l.zid, l.title)
}

pub(crate) fn detail(d: &NoteDetail) -> String {
    let mut out = format!("{}\n", line(&d.note));
    if let Some(source) = &d.note.source {
        let _ = writeln!(out, "source: {source}");
    }
    if !d.note.body.trim().is_empty() {
        out.push('\n');
        out.push_str(d.note.body.trim_end());
        out.push('\n');
    }
    if !d.links.is_empty() || !d.backlinks.is_empty() {
        out.push('\n');
    }
    for l in &d.links {
        out.push_str(&link_line("->", l));
        out.push('\n');
    }
    for l in &d.backlinks {
        out.push_str(&link_line("<-", l));
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// Turn model errors into messages for people: "not found" names the note,
/// validation errors become `field: reason` lines.
pub(crate) fn found<T>(res: std::result::Result<T, ModelError>, zid: &str) -> Result<T> {
    res.map_err(|err| match err {
        ModelError::EntityNotFound => Error::Message(format!("note {zid} not found")),
        err => explain(err),
    })
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    let tags = (!args.tags.is_empty()).then_some(args.tags);
    if args.create {
        let title = args
            .title
            .or(args.quick_title)
            .ok_or_else(|| Error::Message("a title is required: ztk note -c \"Title\"".into()))?;
        let mut body = read_body(args.body)?.unwrap_or_default();
        if args.edit {
            body = edit(&format!("# {title}\n\n{body}"))?;
            body = strip_heading(&body, &title);
        }
        let params = CreateNote {
            title,
            body,
            kind: args.kind,
            source: args.source,
            resource_id: args.resource,
            after: args.after.clone(),
            tags,
        };
        let parent = args.after.unwrap_or_default();
        let note = found(notes::ActiveModel::create(db, params).await, &parent)?;
        out.one(&NoteDetail::load(db, note).await?, detail);
    } else if let Some(zid) = args.update {
        let note = found(notes::Model::find(db, &zid).await, &zid)?;
        let mut body = read_body(args.body)?;
        if args.edit {
            let edited = edit(body.as_deref().unwrap_or(&note.body))?;
            body = Some(edited);
        }
        let params = UpdateNote {
            title: args.title,
            body,
            kind: args.kind,
            source: args.source.into(),
            resource_id: args.resource.into(),
            tags,
        };
        let note = found(note.update_with(db, params).await, &zid)?;
        out.one(&NoteDetail::load(db, note).await?, detail);
    } else if let Some(zid) = args.delete {
        found(notes::Model::find(db, &zid).await, &zid)?
            .remove(db)
            .await?;
        if out.json {
            println!("{}", serde_json::json!({ "deleted": "note", "zid": zid }));
        } else {
            println!("deleted note {zid}");
        }
    } else if let Some(zid) = args.show {
        let note = found(notes::Model::find(db, &zid).await, &zid)?;
        if args.md {
            print!(
                "{}",
                note.to_markdown(&note.tags(db).await?, &note.links(db).await?)
            );
        } else {
            out.one(&NoteDetail::load(db, note).await?, detail);
        }
    } else {
        let found = notes::Entity::listing(
            args.kind.as_deref(),
            tags.as_ref().and_then(|t| t.first()).map(String::as_str),
            args.query.as_deref().or(args.quick_title.as_deref()),
        )
        .limit(args.limit)
        .all(db)
        .await?;
        out.many(&NoteDto::many(db, found).await?, line);
    }
    Ok(())
}

/// `--body -` reads stdin; so does piping into `ztk note -c` without `--body`.
fn read_body(body: Option<String>) -> Result<Option<String>> {
    match body.as_deref() {
        Some("-") => Ok(Some(read_stdin()?)),
        Some(_) => Ok(body),
        None if !std::io::stdin().is_terminal() => {
            let text = read_stdin()?;
            Ok((!text.trim().is_empty()).then_some(text))
        }
        None => Ok(None),
    }
}

fn read_stdin() -> Result<String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| Error::Message(format!("cannot read stdin: {e}")))?;
    Ok(text)
}

/// Open `initial` in `$VISUAL`/`$EDITOR` (default `vi`) and return what was saved.
fn edit(initial: &str) -> Result<String> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into());
    let path = std::env::temp_dir().join(format!("ztk-{}.md", std::process::id()));
    std::fs::write(&path, initial).map_err(|e| Error::Message(e.to_string()))?;
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let status = Command::new(program)
        .args(parts)
        .arg(&path)
        .status()
        .map_err(|e| Error::Message(format!("cannot start editor `{editor}`: {e}")))?;
    let text = std::fs::read_to_string(&path).map_err(|e| Error::Message(e.to_string()));
    let _ = std::fs::remove_file(&path);
    if !status.success() {
        return Err(Error::Message(format!(
            "editor `{editor}` exited with {status}"
        )));
    }
    text
}

/// Drop the `# Title` line the editor template starts with.
fn strip_heading(body: &str, title: &str) -> String {
    body.trim_start()
        .strip_prefix(&format!("# {title}"))
        .unwrap_or(body)
        .trim_start_matches('\n')
        .to_string()
}
