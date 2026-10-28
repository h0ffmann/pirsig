// Areas, projects, resources and Hippos by id, for pickers and labels.
// Views call `refs.dirty()` after changing any of them.
import { all, api } from "./api.js";

let cache = null;

export const refs = {
  async load() {
    if (!cache) {
      const [areas, projects, resources, hippos] = await Promise.all([
        all("/api/areas"),
        all("/api/projects"),
        all("/api/resources"),
        api.get("/api/hippos"),
      ]);
      cache = { areas, projects, resources, hippos };
    }
    return cache;
  },
  dirty() {
    cache = null;
  },
  async options(what) {
    const c = await refs.load();
    const label = { areas: (a) => a.name, projects: (p) => p.name, resources: (r) => r.title, hippos: (x) => x.name }[what];
    return c[what].map((x) => ({ value: x.id, label: label(x) }));
  },
  name(what, id) {
    if (id === null || id === undefined || !cache) return "";
    const x = cache[what]?.find((y) => y.id === id);
    return x ? (x.name ?? x.title) : `#${id}`;
  },
};
