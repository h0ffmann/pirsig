// The Pirsig app: a hash router over the views, the sidebar and quick capture.
import { api } from "./api.js";
import { h, toast } from "./ui.js";
import * as today from "./views/today.js";
import * as para from "./views/para.js";
import * as notes from "./views/notes.js";
import * as slipbox from "./views/slipbox.js";
import * as hippos from "./views/hippos.js";
import * as feeds from "./views/feeds.js";
import * as practice from "./views/practice.js";

const NAV = [
  ["Brain"],
  ["#/", "Today"],
  ["#/notes", "Notes"],
  ["#/slipbox", "Slip-box", "inbox"],
  ["#/hippos", "Hippos"],
  ["PARA"],
  ["#/projects", "Projects"],
  ["#/areas", "Areas"],
  ["#/resources", "Resources"],
  ["#/tasks", "Tasks"],
  ["In and out"],
  ["#/feeds", "Feeds and arXiv"],
  ["Practice"],
  ["#/plasticity", "Plasticity"],
  ["#/ready", "Readiness"],
];

const ROUTES = [
  [/^\/$/, today.view],
  [/^\/notes$/, notes.list],
  [/^\/notes\/new$/, notes.edit],
  [/^\/notes\/([^/]+)\/edit$/, notes.edit],
  [/^\/notes\/([^/]+)$/, notes.show],
  [/^\/slipbox(?:\/(\w+))?$/, slipbox.view],
  [/^\/hippos$/, hippos.list],
  [/^\/hippos\/(\d+)$/, hippos.show],
  [/^\/projects$/, para.projects],
  [/^\/projects\/(\d+)$/, para.project],
  [/^\/areas$/, para.areas],
  [/^\/areas\/(\d+)$/, para.area],
  [/^\/resources$/, para.resources],
  [/^\/resources\/(\d+)$/, para.resource],
  [/^\/tasks$/, para.tasks],
  [/^\/feeds$/, feeds.view],
  [/^\/plasticity$/, practice.plasticity],
  [/^\/ready$/, practice.readiness],
];

function location() {
  const raw = window.location.hash.replace(/^#/, "") || "/";
  const [path, q] = raw.split("?");
  return { path: decodeURI(path) || "/", query: new URLSearchParams(q ?? "") };
}

export function go(hash) {
  if (window.location.hash === hash) render();
  else window.location.hash = hash;
}

let counts = {};
async function sidebar() {
  const { path } = location();
  const section = `#/${path.split("/")[1] ?? ""}`;
  document.getElementById("nav").replaceChildren(
    ...NAV.map(([href, label, count]) => {
      if (!label) return h("div", { class: "group" }, href);
      const on = href === "#/" ? path === "/" : section === href;
      return h("a", { href, class: on ? "on" : null }, label, count && counts[count] ? h("span", { class: "count", "aria-label": `, ${counts[count]} in the ${count}` }, counts[count]) : null);
    }),
  );
}

export async function refreshCounts() {
  try {
    const stats = await api.get("/api/slipbox/stats");
    counts = { inbox: stats.inbox || "" };
  } catch {
    counts = {};
  }
  sidebar();
}

let rendering = 0;
async function render() {
  const turn = ++rendering;
  const { path, query } = location();
  sidebar();
  const view = document.getElementById("view");
  const route = ROUTES.find(([re]) => re.test(path));
  if (!route) {
    view.replaceChildren(h("h1", {}, "Not found"), h("p", {}, h("a", { href: "#/" }, "Back to today")));
    return;
  }
  const args = path.match(route[0]).slice(1).map((a) => (a === undefined ? a : decodeURIComponent(a)));
  view.setAttribute("aria-busy", "true");
  try {
    const node = await route[1]({ args, query, go, reload: render });
    if (turn !== rendering) return;
    view.replaceChildren(node);
    view.scrollTop = 0;
  } catch (e) {
    if (turn !== rendering) return;
    view.replaceChildren(h("h1", {}, "Something went wrong"), h("p", { class: "error" }, e.message),
      h("p", {}, h("a", { href: "#/" }, "Back to today")));
  } finally {
    view.removeAttribute("aria-busy");
  }
}

document.getElementById("capture").addEventListener("submit", async (e) => {
  e.preventDefault();
  const input = e.target.elements.title;
  const title = input.value.trim();
  if (!title) return;
  try {
    const note = await api.post("/api/notes", { title });
    input.value = "";
    toast(`Captured “${note.title}” in the inbox as ${note.zid}`);
    refreshCounts();
    if (location().path === "/" || location().path.startsWith("/slipbox")) render();
  } catch (err) {
    toast(err.message, true);
  }
});

window.addEventListener("hashchange", render);
render();
refreshCounts();
