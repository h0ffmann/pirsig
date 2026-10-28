// The slip-box as a whole: inbox, orphans, the Folgezettel tree, tags, chance.
import { api } from "../api.js";
import { h, head, kind, list, tabs } from "../ui.js";
import { noteRow } from "./notes.js";

const TABS = [
  ["inbox", "Inbox", "#/slipbox/inbox"],
  ["orphans", "Orphans", "#/slipbox/orphans"],
  ["tree", "Tree", "#/slipbox/tree"],
  ["tags", "Tags", "#/slipbox/tags"],
];

export async function view({ args, go }) {
  const tab = args[0] ?? "inbox";
  if (tab === "random") {
    const n = await api.get("/api/slipbox/random");
    go(`#/notes/${encodeURIComponent(n.zid)}`);
    return h("p", {}, "Opening a random note…");
  }
  let body;
  if (tab === "inbox") {
    const notes = await api.get("/api/slipbox/inbox");
    body = [h("p", { class: "sub" }, "Fleeting notes: rewrite each as a permanent note in your words, or delete it."),
      list(notes, (n) => noteRow(n), "Inbox zero.")];
  } else if (tab === "orphans") {
    const notes = await api.get("/api/slipbox/orphans");
    body = [h("p", { class: "sub" }, "Permanent notes with no links in or out: link each to where it belongs."),
      list(notes, (n) => noteRow(n), "No orphans: every permanent note is linked.")];
  } else if (tab === "tree") {
    const tree = await api.get("/api/slipbox/tree");
    body = [h("p", { class: "sub" }, "Every note in Folgezettel order: a continuation sits under the note it continues."),
      h("ul", { class: "list tree" }, tree.map((n) => h("li", { class: `depth-${Math.min(n.depth, 8)}` },
        h("span", { class: "mono muted" }, n.zid),
        h("a", { href: `#/notes/${encodeURIComponent(n.zid)}` }, n.title), kind(n.kind))))];
  } else {
    const stats = await api.get("/api/slipbox/stats");
    body = [h("p", { class: "sub" }, `${stats.tags.length} tags. ${stats.by_kind.map((k) => `${k.count} ${k.name}`).join(", ")}.`),
      h("p", { class: "tagcloud" }, stats.tags.map((t) => h("a", { class: "tag", href: `#/notes?tag=${encodeURIComponent(t.name)}` }, `#${t.name}`, h("span", { class: "muted" }, ` ${t.count}`), " ")))];
  }
  return h("div", {},
    head("Slip-box", h("a", { class: "button", href: "#/slipbox/random" }, "Random note"), h("a", { class: "button primary", href: "#/notes/new" }, "New note")),
    tabs(TABS, tab),
    body);
}
