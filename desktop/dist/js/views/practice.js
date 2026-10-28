// Practice: Plasticity exercises, and how ready the brain is.
import { api } from "../api.js";
import { attempt, button, date, h, head, list, tabs } from "../ui.js";

const DOMAINS = ["math", "abstraction", "logic", "music", "literature", "languages"];

function exercise(stage, domain, onAnswered) {
  let started = performance.now();
  const show = async () => {
    const ex = await attempt(() => api.get("/api/plasticity/next", { domain }));
    if (!ex) return;
    started = performance.now();
    const verdict = h("div", { class: "verdict" });
    const send = async (answer) => {
      const r = await attempt(() => api.post("/api/plasticity/answer", { plast: ex.plast, seed: ex.seed, level: ex.level, answer, millis: Math.round(performance.now() - started) }));
      if (!r) return;
      stage.querySelectorAll("button.choice, input[name=answer]").forEach((b) => { b.disabled = true; });
      verdict.replaceChildren(
        h("p", { class: r.correct ? "s-ready" : "s-missing" }, r.correct ? "Right." : `Not quite: ${r.expected}.`),
        r.explain ? h("p", {}, r.explain) : null,
        h("p", { class: "muted" }, `${r.plast.title}: level ${r.plast.level}, streak ${r.plast.streak}`),
        button("Next exercise", show, "primary"));
      onAnswered();
    };
    const answer = ex.choices.length
      ? h("div", { class: "choices" }, ex.choices.map((c, i) => h("button", { type: "button", class: "choice", onclick: () => send(String.fromCharCode(65 + i)) }, `${String.fromCharCode(65 + i)}. ${c}`)))
      : h("form", { class: "row", onsubmit: (e) => { e.preventDefault(); send(e.target.elements.answer.value); } },
        h("input", { name: "answer", class: "grow", autofocus: true, "aria-label": "Your answer" }), h("button", { type: "submit" }, "Answer"));
    stage.replaceChildren(h("div", { class: "card" },
      h("div", { class: "muted" }, `${ex.plast} · level ${ex.level}`),
      h("div", { class: "prompt" }, ex.prompt),
      answer, verdict));
  };
  return show;
}

export async function plasticity({ query }) {
  const domain = query.get("domain") ?? "";
  const plasts = await api.get("/api/plasticity", { domain });
  const stage = h("div", {});
  const table = h("div", {});
  const paint = (items) => table.replaceChildren(list(items, (p) => h("li", {},
    h("div", { class: "grow" }, h("span", { class: "title" }, p.title), " ", h("span", { class: "pill" }, p.domain),
      p.description ? h("div", { class: "meta" }, p.description) : null),
    h("span", { class: "meta" }, `level ${p.level} · streak ${p.streak}${p.due_at ? ` · due ${date(p.due_at)}` : ""}`)), "No exercises."));
  paint(plasts);
  const start = exercise(stage, domain, async () => paint(await api.get("/api/plasticity", { domain })));
  return h("div", {},
    head("Plasticity", button("Practise", start, "primary")),
    h("p", { class: "sub" }, "Quick exercises that keep the brain supple; the most due comes first."),
    tabs([["", "All", "#/plasticity"], ...DOMAINS.map((d) => [d, d, `#/plasticity?domain=${d}`])], domain),
    stage,
    h("h2", {}, "Exercises"),
    table);
}

export async function readiness() {
  const r = await api.get("/api/readiness");
  const mark = { ready: "✓", partial: "~", missing: "✗" };
  return h("div", {},
    head("Readiness"),
    h("p", { class: "sub" }, `${r.ready} of ${r.total} checks ready: how well the brain follows its methods.`),
    r.sections.map((s) => [
      h("h2", {}, h("span", { class: `s-${s.status}` }, mark[s.status]), " ", s.name),
      h("ul", { class: "list" }, s.checks.map((c) => h("li", {},
        h("span", { class: `s-${c.status}` }, mark[c.status]),
        h("div", { class: "grow" }, h("span", { class: "title" }, c.name), h("div", { class: "meta" }, c.detail)),
        c.fix ? h("code", {}, c.fix) : null))),
    ]));
}
