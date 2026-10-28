// Hippos: rhizomes of notes for minds that wander, with their maps.
import { api } from "../api.js";
import { refs } from "../refs.js";
import { attempt, button, confirm, form, h, head, list, toast } from "../ui.js";

async function editHippo(hippo, after) {
  const saved = await form({
    title: hippo ? `Edit ${hippo.name}` : "New Hippo",
    fields: [
      { name: "name", label: "Name", required: true, autofocus: true },
      { name: "description", label: "Description", type: "textarea" },
      { name: "fixations", label: "Fixations: what the rhizome grows around", type: "list", placeholder: "waves, game theory" },
    ],
    values: hippo ?? {},
    submit: (v) => (hippo ? api.patch(`/api/hippos/${hippo.id}`, v) : api.post("/api/hippos", v)),
  });
  if (saved) {
    refs.dirty();
    toast(hippo ? "Hippo saved" : `Hippo ${saved.name} created`);
    after(saved);
  }
}

async function removeHippo(hippo, after) {
  if (!(await confirm(`Delete the Hippo ${hippo.name}? Its notes stay, out of the rhizome.`))) return;
  if ((await attempt(() => api.del(`/api/hippos/${hippo.id}`), "Hippo deleted")) !== undefined) {
    refs.dirty();
    after();
  }
}

export async function list_({ reload }) {
  const hippos = await api.get("/api/hippos");
  return h("div", {},
    head("Hippos", button("New Hippo", () => editHippo(null, reload), "primary")),
    h("p", { class: "sub" }, "A Hippo is a rhizome of notes: many entry points, no root, linked as you write."),
    list(hippos, (x) => h("li", {},
      h("div", { class: "grow" }, h("a", { class: "title", href: `#/hippos/${x.id}` }, x.name),
        h("div", { class: "meta" }, [x.fixations.length ? `fixations: ${x.fixations.join(", ")}` : null, x.description].filter(Boolean).join(" · "))),
      h("div", { class: "actions" },
        button("Edit", () => editHippo(x, reload), "small"),
        button("Delete", () => removeHippo(x, reload), "small danger"))),
    "No Hippos yet."));
}
export { list_ as list };

const refsList = (notes, nothing) => list(notes, (n) => h("li", {},
  h("span", { class: "mono muted" }, n.zid), h("a", { class: "grow", href: `#/notes/${encodeURIComponent(n.zid)}` }, n.title)), nothing);

export async function show({ args, go, reload }) {
  const id = Number(args[0]);
  const map = await api.get(`/api/hippos/${id}/map`);
  const x = map.hippo;
  return h("div", {},
    h("div", { class: "crumbs" }, h("a", { href: "#/hippos" }, "Hippos")),
    head(x.name,
      h("a", { class: "button primary", href: `#/notes/new?hippo_id=${id}` }, "Grow a note"),
      button("Weave", async () => {
        const r = await attempt(() => api.post(`/api/hippos/${id}/weave`));
        if (r) {
          toast(`Woven ${r.woven} link(s), bridged ${r.bridged}`);
          reload();
        }
      }, "small"),
      button("Edit", () => editHippo(x, reload), "small"),
      button("Delete", () => removeHippo(x, () => go("#/hippos")), "small danger")),
    h("p", { class: "sub" }, `${map.notes} note(s), ${map.links} link(s), ${map.components.length} piece(s)`),
    x.description ? h("p", {}, x.description) : null,
    h("div", { class: "cols" },
      h("section", {},
        h("h2", {}, "Plateaus: notes around each fixation"),
        map.plateaus.length
          ? map.plateaus.map((p) => [h("h3", {}, p.fixation), refsList(p.notes, "Nothing around it yet.")])
          : h("p", { class: "muted" }, "No fixations named: edit the Hippo to add some."),
        h("h2", {}, "Pieces"),
        h("p", { class: "sub" }, map.components.length > 1 ? "Separate pieces: Weave bridges them." : "One piece: a whole rhizome."),
        map.components.map((c, i) => [h("h3", {}, `Piece ${i + 1} (${c.length})`), refsList(c.slice(0, 20), "")])),
      h("section", {},
        h("h2", {}, "Where you left off"),
        refsList(map.resume, "Nothing yet: grow the first note."))));
}
