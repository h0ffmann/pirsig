// Notes: search, one note with its links and backlinks, the editor.
import { api } from "../api.js";
import { markdown } from "../md.js";
import { refs } from "../refs.js";
import { attempt, button, confirm, date, form, h, head, kind, list, pager, toast } from "../ui.js";
import { refreshCounts } from "../main.js";

export const KINDS = ["fleeting", "literature", "permanent", "hub"];
const HASHTAG = /(?:^|(?<=[\s(]))#([\p{L}\p{N}_/-]*\p{L}[\p{L}\p{N}_/-]*)/gu;

function hashtags(body) {
  // tags in code spans and fences do not count, as in ztk itself
  const prose = (body ?? "").replace(/```[\s\S]*?```/g, "").replace(/`[^`\n]*`/g, "");
  return [...prose.matchAll(HASHTAG)].map((m) => m[1].toLowerCase());
}

const noteHref = (zid) => `#/notes/${encodeURIComponent(zid)}`;

export function noteRow(n, extra) {
  return h("li", {},
    h("span", { class: "mono muted" }, n.zid),
    h("div", { class: "grow" },
      h("a", { class: "title", href: noteHref(n.zid) }, n.title), " ", kind(n.kind),
      n.tags?.length ? h("div", { class: "meta" }, n.tags.map((t) => h("a", { class: "tag", href: `#/notes?tag=${encodeURIComponent(t)}` }, `#${t} `))) : null,
      n.context ? h("div", { class: "meta" }, n.context) : null),
    extra ?? h("span", { class: "meta" }, date(n.updated_at)));
}

export async function list_({ query, go }) {
  const q = query.get("q") ?? "";
  const k = query.get("kind") ?? "";
  const tag = query.get("tag") ?? "";
  const page = Number(query.get("page") ?? 1);
  const res = await api.get("/api/notes", { q, kind: k, tag, page, page_size: 50 });
  const to = (over) => {
    const p = new URLSearchParams({ ...(q ? { q } : {}), ...(k ? { kind: k } : {}), ...(tag ? { tag } : {}), ...over });
    for (const [key, v] of [...p]) if (!v) p.delete(key);
    return `#/notes${p.toString() ? `?${p}` : ""}`;
  };
  const search = h("form", {
    class: "row",
    role: "search",
    onsubmit: (e) => {
      e.preventDefault();
      const f = e.target.elements;
      go(to({ q: f.q.value.trim(), kind: f.kind.value, tag: f.tag.value.trim().replace(/^#/, ""), page: "" }));
    },
  },
  h("input", { name: "q", value: q, placeholder: "Search titles and bodies", class: "grow", "aria-label": "Search notes" }),
  h("select", { name: "kind", "aria-label": "Kind" }, h("option", { value: "" }, "every kind"), KINDS.map((x) => h("option", { value: x, selected: x === k }, x))),
  h("input", { name: "tag", value: tag, placeholder: "#tag", size: 14, "aria-label": "Tag" }),
  h("button", { type: "submit" }, "Search"));
  return h("div", {},
    head("Notes", h("a", { class: "button primary", href: "#/notes/new" }, "New note")),
    h("p", { class: "sub" }, `${res.total_items} note(s)${q || k || tag ? " match" : " in the slip-box"}.`),
    search,
    list(res.items, (n) => noteRow(n), "No notes match."),
    pager(res, (p) => go(to({ page: String(p) }))));
}
export { list_ as list };

async function linkTo(zid, after) {
  const done = await form({
    title: "Link to another note",
    fields: [
      { name: "target", label: "Note to link to", type: "note", required: true },
      { name: "context", label: "Why they belong together", type: "textarea", hint: "A link without a reason is a weak link." },
    ],
    ok: "Link",
    submit: (v) => api.post(`/api/notes/${encodeURIComponent(zid)}/links`, v),
  });
  if (done !== undefined) {
    toast("Linked");
    after();
  }
}

async function pullInto(zid) {
  const options = await refs.options("projects");
  if (!options.length) {
    toast("No project to pull it into yet", true);
    return;
  }
  const done = await form({
    title: "Pull into a project",
    fields: [
      { name: "project", label: "Project", type: "select", required: true, numeric: true, options },
      { name: "context", label: "Why the project needs it", type: "textarea" },
    ],
    ok: "Pull in",
    submit: (v) => api.post(`/api/projects/${v.project}/notes`, { zid, context: v.context }),
  });
  if (done !== undefined) toast("Pulled into the project");
}

async function showMarkdown(zid) {
  const text = await api.get(`/api/notes/${encodeURIComponent(zid)}/markdown`);
  const md = typeof text === "string" ? text : JSON.stringify(text, null, 2);
  await form({
    title: "Markdown, as Obsidian gets it",
    fields: [{ name: "md", label: `${zid}.md`, type: "pre" }],
    values: { md },
    ok: "Copy",
    submit: async () => {
      await navigator.clipboard.writeText(md);
      toast("Copied");
      return true;
    },
  });
}

function linkList(items, zid, reload, back) {
  return list(items, (l) => noteRow(l, h("div", { class: "actions" },
    l.auto ? h("span", { class: "pill", title: "from a [[wikilink]] in the body" }, "in body") : null,
    !back && !l.auto ? button("Unlink", async () => {
      if (await confirm(`Remove the link to “${l.title}”?`, "Unlink")) {
        if ((await attempt(() => api.del(`/api/notes/${encodeURIComponent(zid)}/links/${encodeURIComponent(l.zid)}`), "Unlinked")) !== undefined) reload();
      }
    }, "small") : null)),
  back ? "Nothing links here yet." : "No links yet.");
}

export async function show({ args, go, reload }) {
  const zid = args[0];
  const [n] = await Promise.all([api.get(`/api/notes/${encodeURIComponent(zid)}`), refs.load()]);
  const facts = [
    n.source ? ["Source", /^https?:/.test(n.source) ? h("a", { href: n.source, target: "_blank", rel: "noopener" }, n.source) : n.source] : null,
    n.resource_id ? ["Resource", h("a", { href: `#/resources/${n.resource_id}` }, refs.name("resources", n.resource_id))] : null,
    n.hippo_id ? ["Hippo", h("a", { href: `#/hippos/${n.hippo_id}` }, refs.name("hippos", n.hippo_id))] : null,
  ].filter(Boolean);
  return h("div", {},
    h("div", { class: "crumbs" }, h("a", { href: "#/notes" }, "Notes"), " · ", h("span", { class: "mono" }, n.zid)),
    head(n.title,
      kind(n.kind),
      h("a", { class: "button", href: `${noteHref(zid)}/edit` }, "Edit"),
      button("Link…", () => linkTo(zid, reload), "small"),
      h("a", { class: "button small", href: `#/notes/new?after=${encodeURIComponent(zid)}`, title: "File a new note right after this one" }, "Continue"),
      button("Pull into project", () => pullInto(zid), "small"),
      button("Markdown", () => attempt(() => showMarkdown(zid)), "small"),
      button("Delete", async () => {
        if (!(await confirm(`Delete “${n.title}”? Its links go with it.`))) return;
        if ((await attempt(() => api.del(`/api/notes/${encodeURIComponent(zid)}`), "Note deleted")) !== undefined) {
          refreshCounts();
          go("#/notes");
        }
      }, "small danger")),
    h("p", { class: "sub" },
      n.tags.map((t) => h("a", { class: "tag", href: `#/notes?tag=${encodeURIComponent(t)}` }, `#${t} `)),
      ` updated ${date(n.updated_at)}`),
    facts.length ? h("p", {}, facts.map(([k, v], i) => [i ? " · " : "", h("span", { class: "muted" }, `${k}: `), v])) : null,
    h("div", { class: "cols" },
      markdown(n.body || "_(empty)_", n.links),
      h("aside", {},
        h("h3", {}, `Links (${n.links.length})`),
        linkList(n.links, zid, reload, false),
        h("h3", {}, `Backlinks (${n.backlinks.length})`),
        linkList(n.backlinks, zid, reload, true))));
}

export async function edit({ args, query, go }) {
  const zid = args[0];
  const [note, resources, hippos] = await Promise.all([
    zid ? api.get(`/api/notes/${encodeURIComponent(zid)}`) : null,
    refs.options("resources"),
    refs.options("hippos"),
  ]);
  const start = note ?? {
    title: query.get("title") ?? "",
    kind: query.get("kind") ?? "fleeting",
    body: "",
    resource_id: query.get("resource_id") ? Number(query.get("resource_id")) : null,
    hippo_id: query.get("hippo_id") ? Number(query.get("hippo_id")) : null,
    tags: [],
  };
  const inBody = new Set(hashtags(start.body));
  const explicit = (start.tags ?? []).filter((t) => !inBody.has(t));
  const error = h("div", { class: "error", role: "alert" });
  const body = h("textarea", { name: "body", class: "body", "aria-label": "Body (Markdown)" }, start.body);
  const preview = h("div", {});
  const paint = () => preview.replaceChildren(markdown(body.value || "_Nothing yet: write in Markdown, link with [[zid]] or [[Title]], tag with #tag._", note?.links ?? []));
  body.addEventListener("input", paint);
  paint();
  const select = (name, options, value, none) => h("select", { name },
    h("option", { value: "" }, none), options.map((o) => h("option", { value: o.value, selected: o.value === value }, o.label)));
  const save = async (e) => {
    e?.preventDefault();
    error.textContent = "";
    const f = el.elements;
    const values = {
      title: f.title.value.trim(),
      kind: f.kind.value,
      body: body.value,
      source: f.source.value.trim() || null,
      resource_id: f.resource_id.value ? Number(f.resource_id.value) : null,
      hippo_id: f.hippo_id.value ? Number(f.hippo_id.value) : null,
      tags: f.tags.value.split(/[,\s]+/).map((t) => t.replace(/^#/, "").trim()).filter(Boolean),
    };
    try {
      let saved;
      if (note) saved = await api.patch(`/api/notes/${encodeURIComponent(zid)}`, values);
      else {
        const after = query.get("after");
        saved = await api.post("/api/notes", { ...values, ...(after ? { after } : {}) });
      }
      toast(note ? "Saved" : `Filed as ${saved.zid}`);
      refreshCounts();
      go(noteHref(saved.zid));
    } catch (err) {
      error.textContent = err.message;
    }
  };
  const el = h("form", {
    class: "editor-form",
    onsubmit: save,
    onkeydown: (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") save(e);
    },
  },
  h("div", { class: "row" },
    h("input", { name: "title", value: start.title, placeholder: "Title: one idea, in your words", required: true, class: "grow", "aria-label": "Title", autofocus: true }),
    h("select", { name: "kind", "aria-label": "Kind" }, KINDS.map((k) => h("option", { value: k, selected: k === start.kind }, k)))),
  h("div", { class: "row" },
    h("input", { name: "source", value: start.source ?? "", placeholder: "Source (a literature note needs one, or a resource)", class: "grow", "aria-label": "Source" }),
    select("resource_id", resources, start.resource_id, "no resource"),
    select("hippo_id", hippos, start.hippo_id, "no Hippo")),
  h("input", { name: "tags", value: explicit.join(", "), placeholder: "Tags besides the #hashtags in the body", "aria-label": "Tags" }),
  h("div", { class: "editor" }, body, preview),
  error,
  h("div", { class: "buttons" },
    h("a", { class: "button", href: note ? noteHref(zid) : "#/notes" }, "Cancel"),
    h("button", { type: "submit", class: "primary" }, note ? "Save" : "File the note")));
  return h("div", {},
    h("div", { class: "crumbs" }, h("a", { href: "#/notes" }, "Notes"), note ? [" · ", h("a", { href: noteHref(zid) }, note.zid)] : null),
    head(note ? `Edit ${note.title}` : query.get("after") ? `New note after ${query.get("after")}` : "New note"),
    h("p", { class: "sub" }, "⌘S saves. Links: [[zid]] or [[Title]]; tags: #tag."),
    el);
}
