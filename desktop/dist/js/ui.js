// Small DOM toolkit: elements, dialogs, forms, toasts. Text always goes in as
// text nodes, never as HTML.
import { api } from "./api.js";

export function h(tag, attrs = {}, ...children) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs ?? {})) {
    if (v === undefined || v === null || v === false) continue;
    if (k.startsWith("on") && typeof v === "function") el.addEventListener(k.slice(2), v);
    else if (k === "class") el.className = v;
    else if (k === "value") el.value = v;
    else if (k === "checked") el.checked = Boolean(v);
    else el.setAttribute(k, v === true ? "" : v);
  }
  append(el, children);
  return el;
}

function append(el, children) {
  for (const c of children.flat(Infinity)) {
    if (c === undefined || c === null || c === false) continue;
    el.append(c instanceof Node ? c : document.createTextNode(String(c)));
  }
}

export function toast(message, bad = false) {
  const t = h("div", { class: bad ? "toast bad" : "toast" }, message);
  document.getElementById("toasts").append(t);
  setTimeout(() => t.remove(), bad ? 7000 : 3500);
}

// Run an action from a button; errors become a toast.
export async function attempt(fn, done) {
  try {
    const out = await fn();
    if (done) toast(done);
    return out;
  } catch (e) {
    toast(e.message, true);
    return undefined;
  }
}

export function confirm(message, yes = "Delete") {
  return new Promise((resolve) => {
    const dialog = h("dialog", {});
    const close = (v) => {
      dialog.close();
      dialog.remove();
      resolve(v);
    };
    dialog.append(
      h("form", { method: "dialog", onsubmit: (e) => { e.preventDefault(); close(true); } },
        h("p", {}, message),
        h("div", { class: "buttons" },
          h("button", { type: "button", onclick: () => close(false) }, "Cancel"),
          h("button", { type: "submit", class: "primary" }, yes))));
    dialog.addEventListener("cancel", () => close(false));
    document.body.append(dialog);
    dialog.showModal();
  });
}

// A field of a form. `type`: text, textarea, body, select, date, number,
// checkbox, list (comma-separated), note (zid, with a search).
function control(f, value) {
  const name = f.name;
  switch (f.type) {
    case "textarea":
    case "body":
      return h("textarea", { name, rows: f.type === "body" ? 16 : 4, class: f.type === "body" ? "body" : null, required: f.required }, value ?? "");
    case "select":
      return h("select", { name, required: f.required },
        f.required ? null : h("option", { value: "" }, f.none ?? "—"),
        f.options.map((o) => h("option", { value: o.value, selected: String(o.value) === String(value ?? "") }, o.label)));
    case "checkbox":
      return h("input", { type: "checkbox", name, checked: value });
    case "list":
      return h("input", { name, value: (value ?? []).join(", "), placeholder: f.placeholder });
    case "note":
      return notePicker(name, value);
    case "pre":
      return h("pre", { class: "scroll" }, value ?? "");
    default:
      return h("input", { type: f.type ?? "text", name, value: value ?? "", required: f.required, placeholder: f.placeholder, autofocus: f.autofocus });
  }
}

function read(form, fields) {
  const out = {};
  for (const f of fields) {
    const el = form.elements[f.name];
    if (!el) continue;
    if (f.type === "checkbox") out[f.name] = el.checked;
    else if (f.type === "list") out[f.name] = el.value.split(",").map((s) => s.trim()).filter(Boolean);
    else if (f.type === "number" || f.numeric) out[f.name] = el.value === "" ? null : Number(el.value);
    else out[f.name] = el.value === "" ? null : el.value;
  }
  return out;
}

// A modal form. `submit(values)` saves; when it throws, the error shows in
// the form and the form stays open. Resolves to what `submit` returned, or
// undefined when cancelled.
export function form({ title, fields, values = {}, submit, ok = "Save" }) {
  return new Promise((resolve) => {
    const dialog = h("dialog", {});
    const error = h("div", { class: "error", role: "alert" });
    const close = (v) => {
      dialog.close();
      dialog.remove();
      resolve(v);
    };
    const el = h("form", {
      method: "dialog",
      onsubmit: async (e) => {
        e.preventDefault();
        error.textContent = "";
        button.disabled = true;
        try {
          close(await submit(read(el, fields)));
        } catch (err) {
          error.textContent = err.message;
          button.disabled = false;
        }
      },
    });
    const button = h("button", { type: "submit", class: "primary" }, ok);
    el.append(
      h("h2", {}, title),
      ...fields.map((f) =>
        f.type === "checkbox"
          ? h("label", { class: "field check" }, control(f, values[f.name]), h("span", {}, f.label))
          : h("label", { class: "field" }, h("span", {}, f.label), control(f, values[f.name]), f.hint ? h("span", { class: "hint" }, f.hint) : null)),
      error,
      h("div", { class: "buttons" }, h("button", { type: "button", onclick: () => close(undefined) }, "Cancel"), button));
    dialog.append(el);
    dialog.addEventListener("cancel", () => close(undefined));
    document.body.append(dialog);
    dialog.showModal();
    el.querySelector("[autofocus], input, textarea, select")?.focus();
  });
}

// An input for a note's zid that suggests notes by title as you type.
let pickers = 0;
export function notePicker(name, value) {
  const id = `notes-${++pickers}`;
  const list = h("datalist", { id });
  let timer;
  const input = h("input", {
    name,
    value: value ?? "",
    list: id,
    placeholder: "zid, or type to search titles",
    required: true,
    oninput: () => {
      clearTimeout(timer);
      const q = input.value.trim();
      if (q.length < 2) return;
      timer = setTimeout(async () => {
        try {
          const res = await api.get("/api/notes", { q, page_size: 15 });
          list.replaceChildren(...res.items.map((n) => h("option", { value: n.zid }, `${n.title} (${n.kind})`)));
        } catch {
          // suggestions are a nicety
        }
      }, 200);
    },
  });
  return h("span", { class: "picker" }, input, list);
}

export function empty(text) {
  return h("li", { class: "empty" }, text);
}

export function list(items, row, nothing = "Nothing here yet.") {
  return h("ul", { class: "list" }, items.length ? items.map(row) : empty(nothing));
}

export function kind(k) {
  return h("span", { class: `pill k-${k}` }, k);
}

export function date(d) {
  if (!d) return "";
  const t = new Date(d);
  return Number.isNaN(t.getTime()) ? String(d) : t.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}

export function head(title, ...actions) {
  return h("div", { class: "head" }, h("h1", {}, title), ...actions);
}

export function button(label, onclick, cls) {
  return h("button", { type: "button", class: cls, onclick }, label);
}

export function tabs(items, current) {
  return h("nav", { class: "tabs" }, items.map(([key, label, href]) => h("a", { href, class: key === current ? "on" : null }, label)));
}

export function pager(res, go) {
  if (!res || res.total_pages <= 1) return null;
  const prev = button("‹ Prev", () => go(res.page - 1), "small");
  const next = button("Next ›", () => go(res.page + 1), "small");
  prev.disabled = res.page <= 1;
  next.disabled = res.page >= res.total_pages;
  return h("div", { class: "row" }, prev,
    h("span", { class: "muted" }, `page ${res.page} of ${res.total_pages} · ${res.total_items} in all`), next);
}
