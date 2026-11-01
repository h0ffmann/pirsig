#!/usr/bin/env node
// Videos in the brain: see .claude/agents/video-indexer.md for the method. Node's standard
// library only, and yt-dlp for `fetch`.
//
//   videos.mjs fetch URL     brain/Videos/<date>-<slug>/: video.json and the transcript
//   videos.mjs check DIR     coverage of the video, reading time, house style
//   videos.mjs notes DIR     leitura.markdown into psg notes, then back into brain/
//   videos.mjs page          brain/Videos/index.html: every video on one page
//   videos.mjs serve [PORT]  that page, on localhost
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";

const VIDEOS = "brain/Videos";
const PSG = process.env.PSG ?? "target/debug/psg";
const die = (m) => {
  console.error(m);
  process.exit(1);
};

// --- time ---------------------------------------------------------------------------------

// `1:02:03` or `62:03` → seconds.
const seconds = (s) => s.split(":").reduce((acc, n) => acc * 60 + Number(n), 0);
const clock = (t) =>
  [Math.floor(t / 3600), Math.floor(t / 60) % 60, t % 60]
    .map((n) => String(n).padStart(2, "0"))
    .join(":");

// --- fetch --------------------------------------------------------------------------------

function fetchVideo(url) {
  if (!url) die("usage: videos.mjs fetch URL");
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "video-"));
  const out = path.join(tmp, "v.%(ext)s");
  const ytdlp = (...args) =>
    execFileSync("yt-dlp", ["--skip-download", "-o", out, ...args, url], { stdio: ["ignore", "ignore", "inherit"] });
  ytdlp("--write-info-json");
  const info = JSON.parse(fs.readFileSync(path.join(tmp, "v.info.json"), "utf8"));

  // One track, chosen first: asking for several gets a 429 from the translated ones. A person's
  // captions in the video's language, then any person's, then the automatic
  // track of the spoken language. YouTube lists many `-orig` tracks: the first is not the original.
  const manual = Object.keys(info.subtitles ?? {}).filter((l) => l !== "live_chat");
  const spoken = (l) => l.split("-")[0] === info.language?.split("-")[0];
  const sub = manual.find(spoken) ?? manual[0] ??
    Object.keys(info.automatic_captions ?? {}).find((l) => l.endsWith("-orig") && spoken(l));
  if (!sub) die(`no captions in the spoken language (${info.language}): stop, see the policy`);
  ytdlp(manual.includes(sub) ? "--write-subs" : "--write-auto-subs", "--sub-langs", sub, "--sub-format", "json3");
  const file = `v.${sub}.json3`;

  const date = info.upload_date.replace(/(\d{4})(\d\d)(\d\d)/, "$1-$2-$3");
  const slug = info.title.toLowerCase().normalize("NFD").replace(/[^a-z0-9]+/g, " ").trim()
    .split(" ").slice(0, 5).join("-");
  const dir = path.join(VIDEOS, `${date}-${slug}`);
  fs.mkdirSync(path.join(dir, "figuras"), { recursive: true });
  fs.writeFileSync(path.join(dir, "video.json"), `${JSON.stringify({
    id: info.id, url: info.webpage_url, title: info.title, channel: info.channel, date,
    duration: info.duration, language: info.language ?? null, captions: sub,
    captions_kind: manual.includes(sub) ? "manual" : "automatic",
    description: info.description, hub: null, resource: null,
  }, null, 2)}\n`);

  const events = JSON.parse(fs.readFileSync(path.join(tmp, file), "utf8")).events ?? [];
  const lines = new Map();
  for (const e of events) {
    const text = (e.segs ?? []).map((s) => s.utf8 ?? "").join("").replace(/\s+/g, " ").trim();
    if (!text) continue;
    const block = Math.floor((e.tStartMs ?? 0) / 30000) * 30;
    lines.set(block, [...(lines.get(block) ?? []), text]);
  }
  const txt = [...lines].map(([t, l]) => `[${clock(t)}] ${l.join(" ")}`).join("\n");
  fs.writeFileSync(path.join(dir, "transcript.txt"), `${txt}\n`);
  fs.copyFileSync(path.join(tmp, file), path.join(dir, "transcript.json3"));
  fs.rmSync(tmp, { recursive: true });
  console.log(`${dir}: ${info.title} (${clock(info.duration)}), captions ${sub}`);
}

// --- leitura.markdown ---------------------------------------------------------------------

const NOTE = /^### \[k=([a-z0-9-]+) t=([\d:]+)-([\d:]+)\] (.+)$/;

// Parts (`## `) hold notes (`### [k=key t=a-b] Title`). `-> key: why` is a link and never shows.
function parse(dir) {
  const text = fs.readFileSync(path.join(dir, "leitura.markdown"), "utf8");
  const parts = [];
  let title = "";
  let block = null;
  for (const line of text.split("\n")) {
    const note = line.match(NOTE);
    if (line.startsWith("# ")) title = line.slice(2);
    else if (line.startsWith("## ")) {
      block = { title: line.slice(3), body: [], notes: [] };
      parts.push(block);
    } else if (note) {
      block = {
        key: note[1], from: seconds(note[2]), to: seconds(note[3]), title: note[4],
        body: [], links: [],
      };
      parts.at(-1).notes.push(block);
    } else if (line.startsWith("-> ")) {
      const [target, ...why] = line.slice(3).split(": ");
      block.links.push({ target, why: why.join(": ") });
    } else if (line.startsWith("### ")) die(`note without [k= t=]: ${line}`);
    else block?.body.push(line);
  }
  for (const b of parts.flatMap((p) => [p, ...p.notes])) b.body = b.body.join("\n").trim();
  return { title, parts };
}

const readJson = (f) => JSON.parse(fs.readFileSync(f, "utf8"));
const prose = (s) => s.replace(/^!\[fig\]\(.*\)$/gm, "");

// Words and the reading time, by the formula in gramatico's SKILL.md.
function reading(doc) {
  const text = doc.parts
    .flatMap((p) => [p.title, p.body, ...p.notes.flatMap((n) => [n.title, n.body])])
    .map(prose).join("\n").replace(/[*`#]/g, " ");
  const words = text.split(/\s+/).filter((w) => /\p{L}|\d/u.test(w));
  const sentences = text.split(/[.!?:]\s|\n+/).filter((s) => s.trim());
  const long = words.filter((w) => w.replace(/[^\p{L}]/gu, "").length >= 10).length;
  const mean = words.length / sentences.length;
  const longShare = long / words.length;
  let min = words.length / 238;
  if (mean > 20) min *= 1.15;
  if (longShare > 0.25) min *= 1.15;
  return { words: words.length, mean, longShare, minutes: Math.ceil(min) };
}

function check(dir) {
  if (!dir) die("usage: videos.mjs check DIR");
  const video = readJson(path.join(dir, "video.json"));
  const doc = parse(dir);
  const notes = doc.parts.flatMap((p) => p.notes);
  const problems = [];

  // Every second of the video is in a note: no gaps, and to the very end.
  const spans = notes.map((n) => [n.from, n.to, n.key]).sort((a, b) => a[0] - b[0]);
  let covered = 0;
  for (const [from, to, key] of spans) {
    if (from > covered) problems.push(`gap ${clock(covered)}-${clock(from)} before ${key}`);
    if (to <= from) problems.push(`${key}: ends before it starts`);
    covered = Math.max(covered, to);
  }
  if (covered < video.duration - 1) problems.push(`stops at ${clock(covered)} of ${clock(video.duration)}`);

  const keys = new Set(notes.map((n) => n.key));
  if (keys.size !== notes.length) problems.push("repeated note keys");
  for (const n of notes) for (const l of n.links) {
    if (!keys.has(l.target) && !/^#\d/.test(l.target)) problems.push(`${n.key}: unknown link ${l.target}`);
    if (!l.why) problems.push(`${n.key}: link to ${l.target} has no reason`);
  }

  const figures = readJson(path.join(dir, "figuras.json"));
  const used = new Set();
  const body = doc.parts.flatMap((p) => [p.body, ...p.notes.map((n) => n.body)]).join("\n");
  for (const [, id] of body.matchAll(/^!\[fig\]\((.*)\)$/gm)) used.add(id);
  for (const id of used) {
    const f = figures[id];
    if (!f) { problems.push(`figure ${id} has no entry in figuras.json`); continue; }
    for (const field of ["title", "alt", "takeaway", "read", "missing", "evidence"]) {
      if (!f[field]) problems.push(`figure ${id}: no ${field}`);
    }
    if (!fs.existsSync(path.join(dir, "figuras", `${id}.svg`))) problems.push(`figure ${id}: no svg`);
  }
  for (const id of Object.keys(figures)) if (!used.has(id)) problems.push(`figure ${id} is not in the text`);

  // gramatico, the mechanical part: PTW-1, CIE-11. The rest takes a reader.
  const banned = [
    [/ — /g, "travessão em prosa"],
    [/\bvale ressaltar\b|\bcabe destacar\b|\bno âmbito de\b/gi, "enchimento"],
    [/\butiliz\w+/gi, "`usar`, não `utilizar`"],
    [/\b(realizar|efetuar) (a|o) \w+/gi, "verbo pleno (PTW-4)"],
    [/\bo mesmo\b(?= [a-z]+ (é|foi|está))/gi, "`o mesmo` como pronome (PTW-1)"],
  ];
  for (const [re, why] of banned) {
    for (const m of body.matchAll(re)) problems.push(`${why}: ${m[0]}`);
  }

  const r = reading(doc);
  console.log(`${notes.length} notes, ${Object.keys(figures).length} figures, covers ${clock(covered)} of ${clock(video.duration)}`);
  console.log(`Leitura estimada: ~${r.minutes} min (${r.words} palavras; frase média ${r.mean.toFixed(1)}; ${(r.longShare * 100).toFixed(0)}% palavras longas)`);
  for (const p of problems) console.log(`  ${p}`);
  if (problems.length) process.exit(1);
}

// --- notes --------------------------------------------------------------------------------

const psg = (...args) => JSON.parse(execFileSync(PSG, ["--json", ...args], { encoding: "utf8" }));

function notes(dir) {
  if (!dir) die("usage: videos.mjs notes DIR");
  const file = path.join(dir, "video.json");
  const video = readJson(file);
  if (video.hub) die(`already indexed as note ${video.hub}; the notes are the person's now`);
  const doc = parse(dir);
  const areaId = psg("area").find((a) => a.name === "Software")?.id;
  const resource = psg("resource", "-c", "-t", video.title, "--url", video.url, "-k", "video",
    "--citekey", `${video.channel.split(" ").at(-1).toLowerCase()}${video.date.slice(0, 4)}workflow`,
    "--body", `${video.channel}, ${video.date}. Índice: ${dir}/`, ...(areaId ? ["-a", String(areaId)] : []));

  const tags = (more) => ["--tag", "video", "--tag", "agentic-engineering", "--tag", "status/unverified", ...more];
  const hub = psg("note", "-c", "-t", `Vídeo: ${doc.title}`, "-k", "hub", ...tags([]),
    "-b", `${video.channel}, "${video.title}", ${video.date}: ${video.url}\n\n` +
      `Leitura de 30 minutos, figuras e índice: \`just videos\`, ou ${dir}/.`);
  const zids = new Map();
  let previous = null;
  for (const part of doc.parts) {
    const p = psg("note", "-c", "-t", part.title, "-k", "hub", "-a", hub.zid, ...tags([]),
      "-b", prose(part.body).trim() || part.title);
    psg("link", p.zid, hub.zid, "-w", "parte da leitura deste vídeo");
    for (const n of part.notes) {
      const at = `${video.url}&t=${n.from}s`;
      const footer = `Fonte: ${video.channel}, vídeo, [${clock(n.from)}-${clock(n.to)}](${at}). ` +
        "Marca: (v) confere com a legenda automática; a afirmação de David não foi checada. Status: unverified.";
      const body = `${prose(n.body).trim()}\n\n${footer}`;
      const note = psg("note", "-c", "-t", n.title, "-k", "literature", "-a", p.zid, "-r",
        String(resource.id), ...tags([]), "-b", body);
      zids.set(n.key, note.zid);
      if (previous) psg("link", note.zid, previous, "-w", "continua o argumento da nota anterior no vídeo");
      previous = note.zid;
    }
  }
  for (const n of doc.parts.flatMap((p) => p.notes)) for (const l of n.links) {
    psg("link", zids.get(n.key), l.target.startsWith("#") ? l.target.slice(1) : zids.get(l.target), "-w", l.why);
  }
  fs.writeFileSync(file, `${JSON.stringify({ ...video, hub: hub.zid, resource: resource.id }, null, 2)}\n`);
  sync(hub.zid, resource);
  console.log(`hub ${hub.zid}, ${zids.size} notes, resource ${resource.id}`);
}

// A whole fork export renumbers and rewrites every file. Export aside and copy only what is new.
function sync(hub, resource) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "brain-"));
  execFileSync(PSG, ["fork", "export", tmp], { stdio: "ignore" });
  const notesDir = path.join(tmp, "Zettelkasten");
  for (const f of fs.readdirSync(notesDir)) {
    if (new RegExp(`^${hub}([a-z ])`).test(f)) {
      fs.copyFileSync(path.join(notesDir, f), path.join("brain/Zettelkasten", f));
    }
  }
  const resourcesDir = path.join(tmp, "Resources");
  const fresh = fs.readdirSync(resourcesDir)
    .find((f) => fs.readFileSync(path.join(resourcesDir, f), "utf8").includes(`url: "${resource.url}"`));
  const ids = fs.readdirSync("brain/Resources").map((f) =>
    Number(fs.readFileSync(path.join("brain/Resources", f), "utf8").match(/^pirsig_id: (\d+)/m)?.[1] ?? 0));
  const next = fs.readFileSync(path.join(resourcesDir, fresh), "utf8")
    .replace(/^pirsig_id: \d+/m, `pirsig_id: ${Math.max(...ids) + 1}`);
  fs.writeFileSync(path.join("brain/Resources", fresh), next);
  const area = "brain/Areas/Software.md";
  const stem = fresh.replace(/\.md$/, "");
  fs.writeFileSync(area, fs.readFileSync(area, "utf8").replace(
    "## Resources\n\n", `## Resources\n\n- [[Resources/${stem}|${resource.title}]]\n`));
  fs.rmSync(tmp, { recursive: true });
}

// --- page ---------------------------------------------------------------------------------

const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const inline = (s) => esc(s)
  .replace(/`([^`]+)`/g, "<code>$1</code>")
  .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
  .replace(/⚠/g, '<span class="warn" title="não checado">⚠</span>');

function figure(dir, id, figures) {
  const f = figures[id];
  const svg = fs.readFileSync(path.join(dir, "figuras", `${id}.svg`), "utf8").trim();
  return `<figure id="fig-${esc(id)}"><figcaption class="fig-title">${inline(f.title)}</figcaption>
<div class="svg" role="img" aria-label="${esc(f.alt).replace(/"/g, "&quot;")}">${svg}</div>
<details><summary>Como ler esta figura</summary>
<p><strong>Conclusão.</strong> ${inline(f.takeaway)}</p>
<p><strong>Como ler.</strong> ${inline(f.read)}</p>
<p><strong>O que não mostra.</strong> ${inline(f.missing)}</p>
<p><strong>Evidência.</strong> ${inline(f.evidence)}</p></details></figure>`;
}

function blocks(md, dir, figures) {
  const out = [];
  for (const b of md.split(/\n\s*\n/).map((x) => x.trim()).filter(Boolean)) {
    const fig = b.match(/^!\[fig\]\((.*)\)$/);
    if (fig) out.push(figure(dir, fig[1], figures));
    else if (b.startsWith("- ")) {
      out.push(`<ul>${b.split("\n").map((l) => `<li>${inline(l.slice(2))}</li>`).join("")}</ul>`);
    } else out.push(`<p>${inline(b)}</p>`);
  }
  return out.join("\n");
}

function videoSection(dir, open) {
  const video = readJson(path.join(dir, "video.json"));
  const doc = parse(dir);
  const figures = readJson(path.join(dir, "figuras.json"));
  const r = reading(doc);
  const parts = doc.parts.map((p, i) => {
    const notesHtml = p.notes.map((n) => {
      const at = `${video.url}&t=${n.from}s`;
      return `<section class="note"><h4>${inline(n.title)}</h4>
<a class="time" href="${esc(at)}">${clock(n.from)}–${clock(n.to)}</a>
${blocks(n.body, dir, figures)}</section>`;
    }).join("\n");
    return `<details class="part"${i === 0 ? " open" : ""}><summary>${inline(p.title)}</summary>
${blocks(p.body, dir, figures)}
${notesHtml}</details>`;
  }).join("\n");
  return `<details class="video"${open ? " open" : ""}>
<summary><span class="video-title">${inline(video.title)}</span>
<span class="meta">${esc(video.channel)} · ${video.date} · ${clock(video.duration)} · leitura de ~${r.minutes} min · ${Object.keys(figures).length} figuras</span></summary>
<p class="links"><a href="${esc(video.url)}">Abrir o vídeo</a> · nota <code>${esc(String(video.hub ?? "pendente"))}</code>: <code>psg note -s ${esc(String(video.hub ?? ""))}</code></p>
${parts}
</details>`;
}

function page() {
  const dirs = fs.readdirSync(VIDEOS, { withFileTypes: true })
    .filter((e) => e.isDirectory()).map((e) => path.join(VIDEOS, e.name))
    .filter((d) => fs.existsSync(path.join(d, "leitura.markdown")))
    .sort().reverse();
  const css = fs.readFileSync(new URL("./videos.css", import.meta.url), "utf8");
  const html = `<!doctype html>
<html lang="pt-BR"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Vídeos do cérebro</title>
<style>
${css}</style></head>
<body><main>
<h1>Vídeos do cérebro</h1>
<p class="lede">${dirs.length} ${dirs.length === 1 ? "vídeo indexado" : "vídeos indexados"}. Abra um vídeo, depois uma parte.
Página gerada por <code>just videos</code>.</p>
<div class="tools"><button id="open">Abrir tudo</button><button id="close">Recolher tudo</button></div>
${dirs.map((d, i) => videoSection(d, i === 0)).join("\n")}
</main>
<script>
for (const [id, open] of [["open", true], ["close", false]])
  document.getElementById(id).onclick = () =>
    document.querySelectorAll("details.video, details.part").forEach((d) => (d.open = open));
</script></body></html>
`;
  fs.writeFileSync(path.join(VIDEOS, "index.html"), html);
  console.log(`${path.join(VIDEOS, "index.html")}: ${dirs.length} video(s), ${(html.length / 1024).toFixed(0)} KB`);
}

function serve(port = "8765") {
  page();
  http.createServer((_, res) => {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(fs.readFileSync(path.join(VIDEOS, "index.html")));
  }).listen(Number(port), "127.0.0.1", () => console.log(`http://127.0.0.1:${port}/`));
}

const [command, arg] = process.argv.slice(2);
const commands = { fetch: fetchVideo, check, notes, page, serve };
if (!commands[command]) die("usage: videos.mjs fetch URL | check DIR | notes DIR | page | serve [PORT]");
commands[command](arg);
