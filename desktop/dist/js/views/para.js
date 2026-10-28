// PARA: projects, areas, resources (and the archive of each), and tasks.
import { all, api } from "../api.js";
import { refs } from "../refs.js";
import { attempt, button, confirm, date, form, h, head, kind, list, tabs, toast } from "../ui.js";

const STATUSES = ["active", "waiting", "someday", "done"].map((s) => ({ value: s, label: s }));
const KINDS = ["article", "book", "paper", "repo", "video", "web", "other"].map((k) => ({ value: k, label: k }));

// Delete after asking; when the API refuses because of what hangs below
// (an area's projects, a project's tasks), ask again and force it.
async function remove(path, what) {
  if (!(await confirm(`Delete ${what}? This cannot be undone.`))) return false;
  try {
    await api.del(path);
  } catch (e) {
    if (!/force/i.test(e.message)) {
      toast(e.message, true);
      return false;
    }
    if (!(await confirm(`${e.message}\n\nDelete it and everything under it?`, "Delete all"))) return false;
    try {
      await api.del(path, { force: true });
    } catch (err) {
      toast(err.message, true);
      return false;
    }
  }
  refs.dirty();
  toast(`Deleted ${what}`);
  return true;
}

function archiveButton(path, item, after) {
  const archived = Boolean(item.archived_at);
  return button(archived ? "Unarchive" : "Archive", async () => {
    await attempt(() => api.post(`${path}/${archived ? "unarchive" : "archive"}`), archived ? "Back from the archive" : "Archived");
    refs.dirty();
    after();
  }, "small");
}

// ---- areas ---------------------------------------------------------------

const AREA_FIELDS = [
  { name: "name", label: "Name", required: true, autofocus: true },
  { name: "description", label: "Description (the standard you keep)", type: "textarea" },
];

export async function editArea(area, after) {
  const saved = await form({
    title: area ? `Edit ${area.name}` : "New area",
    fields: AREA_FIELDS,
    values: area ?? {},
    submit: (v) => (area ? api.patch(`/api/areas/${area.id}`, v) : api.post("/api/areas", v)),
  });
  if (saved) {
    refs.dirty();
    toast(area ? "Area saved" : `Area ${saved.name} created`);
    after(saved);
  }
}

export async function areas({ query, reload }) {
  const archived = query.get("archived") === "true";
  const items = await all("/api/areas", { archived });
  return h("div", {},
    head("Areas", button("New area", () => editArea(null, reload), "primary")),
    h("p", { class: "sub" }, "Responsibilities with a standard to keep and no end date."),
    tabs([["active", "Active", "#/areas"], ["archive", "Archive", "#/areas?archived=true"]], archived ? "archive" : "active"),
    list(items, (a) => h("li", {},
      h("div", { class: "grow" }, h("a", { class: "title", href: `#/areas/${a.id}` }, a.name), a.description ? h("div", { class: "meta" }, a.description) : null),
      h("div", { class: "actions" },
        button("Edit", () => editArea(a, reload), "small"),
        archiveButton(`/api/areas/${a.id}`, a, reload),
        button("Delete", async () => (await remove(`/api/areas/${a.id}`, `area ${a.name}`)) && reload(), "small danger"))),
    archived ? "The archive is empty." : "No areas yet: add the parts of life you look after."));
}

export async function area({ args, go, reload }) {
  const id = Number(args[0]);
  const [a, projects, resources] = await Promise.all([
    api.get(`/api/areas/${id}`),
    all("/api/projects", { area_id: id }),
    all("/api/resources", { area_id: id }),
    refs.load(),
  ]);
  return h("div", {},
    h("div", { class: "crumbs" }, h("a", { href: "#/areas" }, "Areas")),
    head(a.name,
      button("Edit", () => editArea(a, reload), "small"),
      archiveButton(`/api/areas/${id}`, a, reload),
      button("Delete", async () => (await remove(`/api/areas/${id}`, `area ${a.name}`)) && go("#/areas"), "small danger")),
    a.archived_at ? h("p", { class: "sub" }, `Archived ${date(a.archived_at)}`) : null,
    a.description ? h("p", {}, a.description) : null,
    h("div", { class: "head" }, h("h2", {}, "Projects"), button("New project", () => editProject(null, reload, { area_id: id }), "small")),
    projectList(projects, reload),
    h("div", { class: "head" }, h("h2", {}, "Resources"), button("New resource", () => editResource(null, reload, { area_id: id }), "small")),
    resourceList(resources, reload));
}

// ---- projects ------------------------------------------------------------

async function projectFields() {
  return [
    { name: "name", label: "Name", required: true, autofocus: true },
    { name: "area_id", label: "Area", type: "select", required: true, numeric: true, options: await refs.options("areas") },
    { name: "status", label: "Status", type: "select", required: true, options: STATUSES },
    { name: "due_on", label: "Due on (a project has a deadline)", type: "date" },
    { name: "description", label: "Description (what done looks like)", type: "textarea" },
  ];
}

export async function editProject(project, after, defaults = {}) {
  const fields = await projectFields();
  if (!fields[1].options.length) {
    toast("A project lives in an area: create an area first", true);
    return;
  }
  const saved = await form({
    title: project ? `Edit ${project.name}` : "New project",
    fields,
    values: project ?? { status: "active", ...defaults },
    submit: (v) => (project ? api.patch(`/api/projects/${project.id}`, v) : api.post("/api/projects", v)),
  });
  if (saved) {
    refs.dirty();
    toast(project ? "Project saved" : `Project ${saved.name} created`);
    after(saved);
  }
}

function projectList(items, reload) {
  const sorted = [...items].sort((a, b) => (a.due_on ?? "9999").localeCompare(b.due_on ?? "9999"));
  return list(sorted, (p) => h("li", {},
    h("div", { class: "grow" },
      h("a", { class: "title", href: `#/projects/${p.id}` }, p.name), " ",
      h("span", { class: "pill" }, p.status ?? "active"),
      h("div", { class: "meta" }, [refs.name("areas", p.area_id), p.due_on ? `due ${date(p.due_on)}` : "no deadline"].filter(Boolean).join(" · "))),
    h("div", { class: "actions" },
      button("Edit", () => editProject(p, reload), "small"),
      archiveButton(`/api/projects/${p.id}`, p, reload),
      button("Delete", async () => (await remove(`/api/projects/${p.id}`, `project ${p.name}`)) && reload(), "small danger"))),
  "No projects here.");
}

export async function projects({ query, reload }) {
  const archived = query.get("archived") === "true";
  const [items] = await Promise.all([all("/api/projects", { archived }), refs.load()]);
  return h("div", {},
    head("Projects", button("New project", () => editProject(null, reload), "primary")),
    h("p", { class: "sub" }, "Outcomes with a deadline, each in an area."),
    tabs([["active", "Active", "#/projects"], ["archive", "Archive", "#/projects?archived=true"]], archived ? "archive" : "active"),
    projectList(items, reload));
}

export async function project({ args, go, reload }) {
  const id = Number(args[0]);
  const [p, tasks, pulled, resources] = await Promise.all([
    api.get(`/api/projects/${id}`),
    all("/api/tasks", { project_id: id }),
    api.get(`/api/projects/${id}/notes`),
    all("/api/resources", { project_id: id }),
    refs.load(),
  ]);
  const add = h("form", {
    class: "row",
    onsubmit: async (e) => {
      e.preventDefault();
      const title = e.target.elements.title.value.trim();
      if (!title) return;
      if (await attempt(() => api.post("/api/tasks", { title, project_id: id }))) reload();
    },
  }, h("input", { name: "title", class: "grow", placeholder: "Add a task and press Enter" }), h("button", { type: "submit" }, "Add"));
  return h("div", {},
    h("div", { class: "crumbs" }, h("a", { href: "#/projects" }, "Projects"), " · ", h("a", { href: `#/areas/${p.area_id}` }, refs.name("areas", p.area_id))),
    head(p.name,
      h("span", { class: "pill" }, p.status ?? "active"),
      button("Edit", () => editProject(p, reload), "small"),
      archiveButton(`/api/projects/${id}`, p, reload),
      button("Delete", async () => (await remove(`/api/projects/${id}`, `project ${p.name}`)) && go("#/projects"), "small danger")),
    h("p", { class: "sub" }, p.due_on ? `Due ${date(p.due_on)}` : "No deadline yet", p.archived_at ? ` · archived ${date(p.archived_at)}` : ""),
    p.description ? h("p", {}, p.description) : null,
    h("div", { class: "cols" },
      h("section", {},
        h("h2", {}, `Tasks (${tasks.filter((t) => !t.done).length} open)`),
        add,
        taskList(tasks, reload, false)),
      h("section", {},
        h("div", { class: "head" }, h("h2", {}, "Notes it pulls in"), button("Pull in a note", () => pull(id, reload), "small")),
        list(pulled, (n) => h("li", {},
          h("div", { class: "grow" }, h("a", { class: "title", href: `#/notes/${encodeURIComponent(n.zid)}` }, n.title), " ", kind(n.kind),
            n.context ? h("div", { class: "meta" }, n.context) : null),
          h("div", { class: "actions" }, button("Drop", async () => {
            if (await attempt(() => api.del(`/api/projects/${id}/notes/${encodeURIComponent(n.zid)}`), "Dropped from the project")) reload();
          }, "small"))),
        "No notes pulled in: a project should use what the slip-box knows."),
        h("div", { class: "head" }, h("h2", {}, "Resources"), button("New resource", () => editResource(null, reload, { project_id: id }), "small")),
        resourceList(resources, reload))));
}

async function pull(projectId, after) {
  const done = await form({
    title: "Pull a note into the project",
    fields: [
      { name: "zid", label: "Note", type: "note", required: true },
      { name: "context", label: "Why it belongs here", type: "textarea" },
    ],
    ok: "Pull in",
    submit: (v) => api.post(`/api/projects/${projectId}/notes`, v),
  });
  if (done) {
    toast("Pulled in");
    after();
  }
}

// ---- tasks ---------------------------------------------------------------

async function taskFields() {
  return [
    { name: "title", label: "Task", required: true, autofocus: true },
    { name: "project_id", label: "Project", type: "select", required: true, numeric: true, options: await refs.options("projects") },
    { name: "due_on", label: "Due on", type: "date" },
    { name: "notes", label: "Notes", type: "textarea" },
    { name: "done", label: "Done", type: "checkbox" },
  ];
}

export async function editTask(task, after, defaults = {}) {
  const fields = await taskFields();
  if (!fields[1].options.length) {
    toast("A task belongs to a project: create a project first", true);
    return;
  }
  const saved = await form({
    title: task ? "Edit task" : "New task",
    fields,
    values: task ?? defaults,
    submit: (v) => (task ? api.patch(`/api/tasks/${task.id}`, v) : api.post("/api/tasks", v)),
  });
  if (saved) {
    toast(task ? "Task saved" : "Task added");
    after(saved);
  }
}

function taskList(tasks, reload, showProject = true) {
  const sorted = [...tasks].sort((a, b) => Number(a.done) - Number(b.done) || (a.due_on ?? "9999").localeCompare(b.due_on ?? "9999") || a.id - b.id);
  return list(sorted, (t) => h("li", { class: t.done ? "done" : null },
    h("input", {
      type: "checkbox",
      checked: t.done,
      "aria-label": `Done: ${t.title}`,
      onchange: async (e) => {
        if (await attempt(() => api.patch(`/api/tasks/${t.id}`, { done: e.target.checked }))) reload();
        else e.target.checked = t.done;
      },
    }),
    h("div", { class: "grow" },
      h("span", { class: "title" }, t.title),
      h("div", { class: "meta" }, [showProject ? refs.name("projects", t.project_id) : null, t.due_on ? `due ${date(t.due_on)}` : null, t.notes].filter(Boolean).join(" · "))),
    h("div", { class: "actions" },
      button("Edit", () => editTask(t, reload), "small"),
      button("Delete", async () => {
        if (await confirm(`Delete the task “${t.title}”?`)) {
          if ((await attempt(() => api.del(`/api/tasks/${t.id}`), "Task deleted")) !== undefined) reload();
        }
      }, "small danger"))),
  "No tasks.");
}

export async function tasks({ query, reload }) {
  const done = query.get("done");
  const [items] = await Promise.all([all("/api/tasks", { done: done ?? false }), refs.load()]);
  return h("div", {},
    head("Tasks", button("New task", () => editTask(null, reload), "primary")),
    tabs([["open", "Open", "#/tasks"], ["done", "Done", "#/tasks?done=true"]], done === "true" ? "done" : "open"),
    taskList(items, reload));
}

// ---- resources -----------------------------------------------------------

async function resourceFields() {
  return [
    { name: "title", label: "Title", required: true, autofocus: true },
    { name: "url", label: "URL", type: "url" },
    { name: "kind", label: "Kind", type: "select", options: KINDS },
    { name: "area_id", label: "Area", type: "select", numeric: true, options: await refs.options("areas"), none: "no area" },
    { name: "project_id", label: "Project", type: "select", numeric: true, options: await refs.options("projects"), none: "no project" },
    { name: "citekey", label: "Citekey (BibTeX)", placeholder: "vaswani2017attention" },
    { name: "body", label: "Notes on it", type: "textarea" },
  ];
}

export async function editResource(resource, after, defaults = {}) {
  const saved = await form({
    title: resource ? `Edit ${resource.title}` : "New resource",
    fields: await resourceFields(),
    values: resource ?? defaults,
    submit: (v) => (resource ? api.patch(`/api/resources/${resource.id}`, v) : api.post("/api/resources", v)),
  });
  if (saved) {
    refs.dirty();
    toast(resource ? "Resource saved" : "Resource added");
    after(saved);
  }
}

function filed(r) {
  if (r.project_id) return refs.name("projects", r.project_id);
  if (r.area_id) return refs.name("areas", r.area_id);
  return "unfiled";
}

function resourceList(items, reload) {
  return list(items, (r) => h("li", {},
    h("div", { class: "grow" },
      h("a", { class: "title", href: `#/resources/${r.id}` }, r.title), " ",
      r.kind ? h("span", { class: "pill" }, r.kind) : null,
      h("div", { class: "meta" }, [filed(r), r.citekey ? `@${r.citekey}` : null].filter(Boolean).join(" · "))),
    h("div", { class: "actions" },
      r.url ? h("a", { class: "button small", href: r.url, target: "_blank", rel: "noopener" }, "Open") : null,
      button("Edit", () => editResource(r, reload), "small"),
      button("Delete", async () => (await remove(`/api/resources/${r.id}`, `resource ${r.title}`)) && reload(), "small danger"))),
  "No resources.");
}

export async function resources({ query, reload, go }) {
  const archived = query.get("archived") === "true";
  const [items] = await Promise.all([all("/api/resources", { archived }), refs.load()]);
  const text = query.get("q") ?? "";
  const kindFilter = query.get("kind") ?? "";
  const shown = items.filter((r) => (!kindFilter || r.kind === kindFilter)
    && (!text || `${r.title} ${r.url ?? ""} ${r.citekey ?? ""}`.toLowerCase().includes(text.toLowerCase())));
  const filter = h("form", {
    class: "row",
    onsubmit: (e) => {
      e.preventDefault();
      const f = e.target.elements;
      const q = new URLSearchParams({ ...(archived ? { archived: "true" } : {}), ...(f.q.value ? { q: f.q.value } : {}), ...(f.kind.value ? { kind: f.kind.value } : {}) });
      go(`#/resources${q.toString() ? `?${q}` : ""}`);
    },
  },
  h("input", { name: "q", value: text, placeholder: "Filter by title, URL or citekey", class: "grow" }),
  h("select", { name: "kind" }, h("option", { value: "" }, "every kind"), KINDS.map((k) => h("option", { value: k.value, selected: k.value === kindFilter }, k.label))),
  h("button", { type: "submit" }, "Filter"));
  return h("div", {},
    head("Resources", button("New resource", () => editResource(null, reload), "primary")),
    h("p", { class: "sub" }, `What you read, watch and use: ${shown.length} of ${items.length}.`),
    tabs([["active", "Active", "#/resources"], ["archive", "Archive", "#/resources?archived=true"]], archived ? "archive" : "active"),
    filter,
    resourceList(shown, reload));
}

export async function resource({ args, go, reload }) {
  const id = Number(args[0]);
  const [r] = await Promise.all([api.get(`/api/resources/${id}`), refs.load()]);
  const facts = [
    ["Kind", r.kind],
    ["URL", r.url ? h("a", { href: r.url, target: "_blank", rel: "noopener" }, r.url) : null],
    ["Filed in", r.project_id ? h("a", { href: `#/projects/${r.project_id}` }, refs.name("projects", r.project_id))
      : r.area_id ? h("a", { href: `#/areas/${r.area_id}` }, refs.name("areas", r.area_id)) : "nowhere yet"],
    ["Citekey", r.citekey ? h("code", {}, r.citekey) : null],
    ["From", r.external_id ? h("code", {}, r.external_id) : null],
    ["Added", date(r.created_at)],
    ["Archived", r.archived_at ? date(r.archived_at) : null],
  ].filter(([, v]) => v);
  return h("div", {},
    h("div", { class: "crumbs" }, h("a", { href: "#/resources" }, "Resources")),
    head(r.title,
      h("a", { class: "button", href: `#/notes/new?resource_id=${id}&kind=literature` }, "Write a literature note"),
      button("Edit", () => editResource(r, reload), "small"),
      archiveButton(`/api/resources/${id}`, r, reload),
      button("Delete", async () => (await remove(`/api/resources/${id}`, `resource ${r.title}`)) && go("#/resources"), "small danger")),
    h("ul", { class: "list" }, facts.map(([k, v]) => h("li", {}, h("span", { class: "muted label" }, k), h("span", { class: "grow" }, v)))),
    r.body ? [h("h2", {}, "Notes on it"), h("p", {}, r.body)] : null);
}
