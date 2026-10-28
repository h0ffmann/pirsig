// What comes in: RSS feeds and arXiv papers, filed as resources.
import { api } from "../api.js";
import { refs } from "../refs.js";
import { attempt, button, confirm, date, form, h, head, list, toast } from "../ui.js";

async function filing() {
  return [
    { name: "area_id", label: "Area", type: "select", numeric: true, options: await refs.options("areas"), none: "no area" },
    { name: "project_id", label: "Project", type: "select", numeric: true, options: await refs.options("projects"), none: "no project" },
  ];
}

async function addFeed(after) {
  const saved = await form({
    title: "Subscribe to a feed",
    fields: [{ name: "url", label: "RSS or Atom URL", type: "url", required: true, autofocus: true }, ...(await filing())],
    ok: "Subscribe",
    submit: (v) => api.post("/api/feeds", v),
  });
  if (saved) {
    toast("Subscribed: pull it to fetch its entries");
    after();
  }
}

async function searchArxiv(e, results) {
  e.preventDefault();
  const q = e.target.elements.q.value.trim();
  if (!q) return;
  results.replaceChildren(h("p", { class: "muted" }, "Searching arXiv…"));
  const papers = await attempt(() => api.get("/api/ingest/arxiv/search", { q, max: 20 }));
  if (!papers) {
    results.replaceChildren();
    return;
  }
  const picked = new Set();
  const ingest = button("Ingest selected", async () => {
    const opts = await form({
      title: `Ingest ${picked.size} paper(s)`,
      fields: [...(await filing()), { name: "note", label: "Also put a “Read:” note in the inbox", type: "checkbox" }],
      values: { note: true },
      ok: "Ingest",
      submit: (v) => api.post("/api/ingest/arxiv", { ids: [...picked], ...v }),
    });
    if (opts) {
      refs.dirty();
      toast(`Filed ${opts.length} paper(s) as resources`);
    }
  }, "primary");
  ingest.disabled = true;
  results.replaceChildren(
    h("div", { class: "head" }, h("h2", {}, `${papers.length} paper(s)`), ingest),
    list(papers, (p) => h("li", {},
      h("input", {
        type: "checkbox",
        "aria-label": `Select ${p.title}`,
        onchange: (ev) => {
          if (ev.target.checked) picked.add(p.id);
          else picked.delete(p.id);
          ingest.disabled = picked.size === 0;
        },
      }),
      h("div", { class: "grow" },
        h("a", { class: "title", href: `https://arxiv.org/abs/${p.id}`, target: "_blank", rel: "noopener" }, p.title),
        h("div", { class: "meta" }, [p.authors.slice(0, 4).join(", ") + (p.authors.length > 4 ? " et al." : ""), p.published ? date(p.published) : null, p.categories[0]].filter(Boolean).join(" · ")),
        h("details", {}, h("summary", { class: "meta" }, "Abstract"), h("p", {}, p.abstract)))),
    "Nothing found."));
}

export async function view({ reload }) {
  const [feeds] = await Promise.all([api.get("/api/feeds"), refs.load()]);
  const results = h("div", {});
  return h("div", {},
    head("Feeds and arXiv", button("Subscribe", () => addFeed(reload), "primary")),
    h("h2", {}, "RSS feeds"),
    list(feeds, (f) => h("li", {},
      h("div", { class: "grow" },
        h("span", { class: "title" }, f.title ?? f.url),
        h("div", { class: "meta" }, [f.title ? f.url : null, refs.name("projects", f.project_id) || refs.name("areas", f.area_id) || "unfiled",
          f.last_fetched_at ? `pulled ${date(f.last_fetched_at)}` : "never pulled"].filter(Boolean).join(" · "))),
      h("div", { class: "actions" },
        button("Pull", async (e) => {
          e.target.disabled = true;
          const r = await attempt(() => api.post(`/api/feeds/${f.id}/pull`));
          e.target.disabled = false;
          if (r) {
            refs.dirty();
            toast(`${r.title ?? "Feed"}: ${r.new} new, ${r.updated} updated`);
            reload();
          }
        }, "small"),
        button("Unsubscribe", async () => {
          if (await confirm(`Unsubscribe from ${f.title ?? f.url}? Its resources stay.`, "Unsubscribe")) {
            if ((await attempt(() => api.del(`/api/feeds/${f.id}`), "Unsubscribed")) !== undefined) reload();
          }
        }, "small danger"))),
    "No feeds: subscribe to the blogs and journals you follow."),
    h("h2", {}, "arXiv"),
    h("form", { class: "row", role: "search", onsubmit: (e) => searchArxiv(e, results) },
      h("input", { name: "q", class: "grow", placeholder: "Search arXiv: rogue waves, mean field games…", "aria-label": "Search arXiv" }),
      h("button", { type: "submit" }, "Search")),
    results);
}
