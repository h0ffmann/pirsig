// Note bodies as DOM: the Markdown a slip-box uses (headings, paragraphs,
// lists, quotes, code, emphasis, links), plus `[[wikilinks]]` and `#tags`.
// Everything is built as nodes from text; no HTML from a note is ever parsed.
import { h } from "./ui.js";

// `[[target]]`, `[[target|alias]]`, `[[target#heading]]`: the note it names,
// by zid or title, among the note's resolved links.
function wikilink(raw, links) {
  const [target, alias] = raw.split("|");
  const name = target.split("#")[0].trim();
  const key = name.toLowerCase();
  const hit = links.find((l) => l.zid === name || l.title.toLowerCase() === key);
  const text = (alias ?? target).trim();
  return hit
    ? h("a", { class: "wikilink", href: `#/notes/${encodeURIComponent(hit.zid)}`, title: hit.title }, text)
    : h("a", { class: "wikilink missing", href: `#/notes?q=${encodeURIComponent(name)}`, title: "no note by that name yet" }, text);
}

const INLINE = /(`[^`\n]+`)|(\[\[[^\]\n]+\]\])|(\*\*[^*\n]+\*\*)|(\*[^*\s][^*\n]*\*|_[^_\s][^_\n]*_)|(\[[^\]\n]+\]\([^)\s]+\))|((?:^|(?<=[\s(]))#[\p{L}\p{N}_/-]*\p{L}[\p{L}\p{N}_/-]*)|(https?:\/\/[^\s<>)]+)/gu;

function safeUrl(url) {
  return /^(https?:|mailto:|#)/i.test(url) ? url : null;
}

export function inline(text, links = []) {
  const out = [];
  let last = 0;
  for (const m of text.matchAll(INLINE)) {
    if (m.index > last) out.push(text.slice(last, m.index));
    const [all, code, wiki, strong, em, link, tag, url] = m;
    if (code) out.push(h("code", {}, code.slice(1, -1)));
    else if (wiki) out.push(wikilink(wiki.slice(2, -2), links));
    else if (strong) out.push(h("strong", {}, inline(strong.slice(2, -2), links)));
    else if (em) out.push(h("em", {}, inline(em.slice(1, -1), links)));
    else if (link) {
      const [, label, href] = link.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
      const safe = safeUrl(href);
      out.push(safe ? h("a", { href: safe, target: safe.startsWith("#") ? null : "_blank", rel: "noopener" }, inline(label, links)) : label);
    } else if (tag) out.push(h("a", { class: "tag", href: `#/notes?tag=${encodeURIComponent(tag.slice(1).toLowerCase())}` }, tag));
    else if (url) out.push(h("a", { href: url, target: "_blank", rel: "noopener" }, url));
    else out.push(all);
    last = m.index + all.length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

export function markdown(body, links = []) {
  const root = h("div", { class: "note-body" });
  const lines = (body ?? "").replace(/\r\n/g, "\n").split("\n");
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i++;
      continue;
    }
    const fence = line.match(/^```(\S*)/);
    if (fence) {
      const code = [];
      i++;
      while (i < lines.length && !lines[i].startsWith("```")) code.push(lines[i++]);
      i++;
      root.append(h("pre", {}, h("code", {}, code.join("\n"))));
      continue;
    }
    const heading = line.match(/^(#{1,6})\s+(.*)$/);
    if (heading) {
      root.append(h(`h${Math.min(heading[1].length + 1, 6)}`, {}, inline(heading[2], links)));
      i++;
      continue;
    }
    if (/^(-{3,}|\*{3,})\s*$/.test(line)) {
      root.append(h("hr"));
      i++;
      continue;
    }
    if (/^>\s?/.test(line)) {
      const quote = [];
      while (i < lines.length && /^>\s?/.test(lines[i])) quote.push(lines[i++].replace(/^>\s?/, ""));
      root.append(h("blockquote", {}, [...markdown(quote.join("\n"), links).childNodes]));
      continue;
    }
    const bullet = /^\s*([-*+]|\d+[.)])\s+/;
    if (bullet.test(line)) {
      const ordered = /^\s*\d/.test(line);
      const items = [];
      while (i < lines.length && bullet.test(lines[i])) {
        const item = lines[i].replace(bullet, "");
        const task = item.match(/^\[([ xX])\]\s+(.*)$/);
        items.push(task
          ? h("li", {}, h("input", { type: "checkbox", disabled: true, checked: task[1] !== " " }), " ", inline(task[2], links))
          : h("li", {}, inline(item, links)));
        i++;
      }
      root.append(h(ordered ? "ol" : "ul", {}, items));
      continue;
    }
    const para = [];
    while (i < lines.length && lines[i].trim() && !/^(```|#{1,6}\s|>|\s*([-*+]|\d+[.)])\s)/.test(lines[i])) para.push(lines[i++]);
    if (!para.length) para.push(lines[i++]);
    root.append(h("p", {}, inline(para.join(" "), links)));
  }
  return root;
}
