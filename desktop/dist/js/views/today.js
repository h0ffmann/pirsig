// Today: what the brain holds, what is due, what waits in the inbox.
import { all, api } from "../api.js";
import { refs } from "../refs.js";
import { date, h, head, list } from "../ui.js";
import { noteRow } from "./notes.js";

export async function view() {
  const [stats, ready, inbox, projects, tasks] = await Promise.all([
    api.get("/api/slipbox/stats"),
    api.get("/api/readiness"),
    api.get("/api/slipbox/inbox"),
    all("/api/projects"),
    all("/api/tasks", { done: false }),
    refs.load(),
  ]);
  const card = (label, value, href) => h("a", { class: "card", href }, h("div", { class: "muted" }, label), h("div", { class: "stat" }, value));
  const active = projects.filter((p) => (p.status ?? "active") === "active")
    .sort((a, b) => (a.due_on ?? "9999").localeCompare(b.due_on ?? "9999"));
  const soon = tasks.filter((t) => t.due_on).sort((a, b) => a.due_on.localeCompare(b.due_on)).slice(0, 8);
  return h("div", {},
    head("Today"),
    h("div", { class: "grid" },
      card("Notes", stats.notes, "#/notes"),
      card("Links", stats.links, "#/slipbox/tree"),
      card("Inbox", stats.inbox, "#/slipbox/inbox"),
      card("Orphans", stats.orphans, "#/slipbox/orphans"),
      card("Readiness", `${ready.ready}/${ready.total}`, "#/ready")),
    h("div", { class: "cols" },
      h("section", {},
        h("h2", {}, "Inbox: fleeting notes to process"),
        list(inbox.slice(0, 12), (n) => noteRow(n), "Inbox zero. Capture a thought from the sidebar."),
        inbox.length > 12 ? h("p", {}, h("a", { href: "#/slipbox/inbox" }, `All ${inbox.length} in the inbox`)) : null),
      h("section", {},
        h("h2", {}, "Active projects"),
        list(active.slice(0, 10), (p) => h("li", {},
          h("div", { class: "grow" }, h("a", { class: "title", href: `#/projects/${p.id}` }, p.name),
            h("div", { class: "meta" }, refs.name("areas", p.area_id))),
          h("span", { class: "meta" }, p.due_on ? date(p.due_on) : "")), "No active projects."),
        h("h2", {}, "Tasks with a date"),
        list(soon, (t) => h("li", {},
          h("div", { class: "grow" }, h("span", { class: "title" }, t.title),
            h("div", { class: "meta" }, h("a", { href: `#/projects/${t.project_id}` }, refs.name("projects", t.project_id)))),
          h("span", { class: "meta" }, date(t.due_on))), "Nothing dated."))));
}
