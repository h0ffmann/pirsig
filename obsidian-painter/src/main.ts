// obsidian-painter: the Obsidian side of a Rust core (crates/obsidian-painter).
//
// Three features, after akitaroh/obsidian-mermaid-maker's code-block approach:
//   ```desmos   a Desmos graph state or LaTeX lines, drawn offline as SVG
//   ```archify  an archify JSON IR, drawn by archify into an HTML page and
//               shown in a shadow root; "Compose note as HTML page" joins them
//   "Export note to PDF": pandoc, with every figure rasterized sharply
// This file only touches Obsidian and the machine (files, node, pandoc);
// everything else is painter.call(...).
import { execFile } from 'child_process';
import { promises as fs } from 'fs';
import * as os from 'os';
import * as path from 'path';
import {
  App,
  FileSystemAdapter,
  Notice,
  Plugin,
  PluginSettingTab,
  Setting,
  TFile,
} from 'obsidian';
import wasm from '../build/painter.wasm';
import { Painter } from './painter';

interface Settings {
  pandoc: string;
  pdfEngine: string;
  pandocArgs: string;
  node: string;
  archifyScript: string;
  archifyCss: string;
  scale: number;
}

const DEFAULTS: Settings = {
  pandoc: 'pandoc',
  pdfEngine: 'xelatex',
  pandocArgs: '',
  node: 'node',
  archifyScript: '',
  archifyCss: '',
  scale: 3,
};

type Block = { id: string; lang: string; source: string };
type Fragment = { css: string; svg: string };

function run(cmd: string, args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    execFile(
      cmd,
      args,
      { env: { ...process.env, ARCHIFY_UPDATE_CHECK_DISABLED: '1' }, maxBuffer: 64 << 20 },
      (err, stdout, stderr) => (err ? reject(new Error(`${cmd}: ${stderr || err.message}`)) : resolve(stdout)),
    );
  });
}

/** Parse SVG text into an element without innerHTML (DOMParser runs no scripts). */
function svgElement(svg: string): Element | null {
  const root = new DOMParser().parseFromString(svg, 'image/svg+xml').documentElement;
  return root.nodeName.toLowerCase() === 'svg' ? root : null;
}

/** An SVG as PNG bytes, drawn at its explicit width and height. */
async function rasterize(svg: string, width: number, height: number): Promise<Uint8Array> {
  const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
  try {
    const img = new Image(width, height);
    img.src = url;
    await img.decode();
    const canvas = document.createElement('canvas');
    canvas.width = Math.ceil(width);
    canvas.height = Math.ceil(height);
    const g = canvas.getContext('2d');
    if (!g) throw new Error('no canvas');
    g.fillStyle = '#ffffff';
    g.fillRect(0, 0, canvas.width, canvas.height);
    g.drawImage(img, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise<Blob | null>((ok) => canvas.toBlob(ok, 'image/png'));
    if (!blob) throw new Error('could not rasterize the figure');
    return new Uint8Array(await blob.arrayBuffer());
  } finally {
    URL.revokeObjectURL(url);
  }
}

export default class PainterPlugin extends Plugin {
  settings: Settings = { ...DEFAULTS };
  painter!: Painter;

  async onload() {
    this.settings = { ...DEFAULTS, ...(await this.loadData()) };
    this.painter = await Painter.load(wasm);
    this.addSettingTab(new PainterSettings(this.app, this));

    this.registerMarkdownCodeBlockProcessor('desmos', (source, el) => {
      this.show(el, () => {
        const r = this.painter.call<{ svg: string; skipped: string[] }>('desmos', { source });
        const svg = svgElement(r.svg);
        if (svg) el.createDiv({ cls: 'painter-desmos-block' }).appendChild(svg);
        if (r.skipped.length) el.createDiv({ cls: 'painter-note', text: `not drawn: ${r.skipped.join('; ')}` });
      });
    });

    this.registerMarkdownCodeBlockProcessor('archify', async (source, el) => {
      el.createDiv({ cls: 'painter-note', text: 'archify: drawing…' });
      await this.show(el, async () => {
        const f = await this.archify(source);
        el.empty();
        const host = el.createDiv({ cls: 'painter-archify-block' });
        const root = host.attachShadow({ mode: 'open' });
        const style = document.createElement('style');
        style.textContent = `svg{max-width:100%;height:auto}${f.css}`;
        root.appendChild(style);
        const svg = svgElement(f.svg);
        if (svg) root.appendChild(svg);
      });
    });

    this.addCommand({
      id: 'export-pdf',
      name: 'Export note to PDF (pandoc)',
      checkCallback: (checking) => this.withNote(checking, (f) => this.exportPdf(f)),
    });
    this.addCommand({
      id: 'compose-html',
      name: 'Compose note as HTML page',
      checkCallback: (checking) => this.withNote(checking, (f) => this.composeHtml(f)),
    });
  }

  private withNote(checking: boolean, act: (f: TFile) => Promise<void>): boolean {
    const file = this.app.workspace.getActiveFile();
    if (!file || file.extension !== 'md') return false;
    if (!checking) {
      act(file).catch((e: Error) => new Notice(`Painter: ${e.message}`, 10000));
    }
    return true;
  }

  private async show(el: HTMLElement, draw: () => void | Promise<void>) {
    try {
      await draw();
    } catch (e) {
      el.empty();
      el.createEl('pre', { cls: 'painter-error', text: `Painter: ${(e as Error).message}` });
    }
  }

  private base(): string {
    const a = this.app.vault.adapter;
    if (!(a instanceof FileSystemAdapter)) throw new Error('Painter needs the desktop app');
    return a.getBasePath();
  }

  private cache(): string {
    return path.join(this.base(), this.app.vault.configDir, 'plugins', this.manifest.id, 'cache');
  }

  /** The archify page for `ir`, drawn once and kept by content. */
  private async archify(ir: string): Promise<Fragment> {
    const script = this.settings.archifyScript;
    if (!script) {
      throw new Error('set the archify script in the settings (vendor/archify/archify/bin/archify.mjs after `just figure-init`)');
    }
    const id = this.painter.call<string>('id', { lang: 'archify', source: ir });
    const dir = path.join(this.cache(), id);
    const page = path.join(dir, 'page.html');
    let html: string;
    try {
      html = await fs.readFile(page, 'utf8');
    } catch {
      await fs.mkdir(dir, { recursive: true });
      const src = path.join(dir, 'source.json');
      await fs.writeFile(src, ir);
      const args = this.painter.call<string[]>('archify_args', { ir, script, src, page, receipts: path.join(dir, 'receipts') });
      await run(this.settings.node, args);
      html = await fs.readFile(page, 'utf8');
    }
    const extra = this.settings.archifyCss ? await fs.readFile(this.settings.archifyCss, 'utf8') : '';
    return this.painter.call<Fragment>('archify', { page: html, extra_css: extra });
  }

  /** Every figure of a note, as a fragment (Desmos figures have no CSS). */
  private async figures(markdown: string): Promise<Map<string, Fragment>> {
    const out = new Map<string, Fragment>();
    for (const b of this.painter.call<Block[]>('blocks', { markdown })) {
      if (b.lang === 'desmos') {
        out.set(b.id, { css: '', svg: this.painter.call<{ svg: string }>('desmos', { source: b.source }).svg });
      } else {
        out.set(b.id, await this.archify(b.source));
      }
    }
    return out;
  }

  private async exportPdf(file: TFile) {
    new Notice('Painter: writing the PDF…');
    const markdown = await this.app.vault.read(file);
    const tmp = await fs.mkdtemp(path.join(os.tmpdir(), 'painter-'));
    const images: Record<string, string> = {};
    for (const [id, f] of await this.figures(markdown)) {
      const one = this.painter.call<string>('standalone', f);
      const s = this.painter.call<{ svg: string; width: number; height: number }>('sized', { svg: one, scale: this.settings.scale });
      const png = path.join(tmp, `${id}.png`);
      await fs.writeFile(png, await rasterize(s.svg, s.width, s.height));
      images[id] = png;
    }
    const input = path.join(tmp, 'note.md');
    await fs.writeFile(input, this.painter.call<string>('prepare', { markdown, target: 'pdf', images }));
    const noteDir = path.join(this.base(), file.parent?.path ?? '');
    const output = path.join(noteDir, `${file.basename}.pdf`);
    const args = this.painter.call<string[]>('pdf_args', {
      input,
      output,
      engine: this.settings.pdfEngine,
      resource_path: [noteDir, this.base()],
      extra: this.settings.pandocArgs.split(/\s+/).filter(Boolean),
    });
    await run(this.settings.pandoc, args);
    await fs.rm(tmp, { recursive: true, force: true });
    new Notice(`Painter: ${path.relative(this.base(), output)}`);
  }

  private async composeHtml(file: TFile) {
    const markdown = await this.app.vault.read(file);
    const figures = Object.fromEntries(await this.figures(markdown));
    const tmp = await fs.mkdtemp(path.join(os.tmpdir(), 'painter-'));
    const input = path.join(tmp, 'note.md');
    await fs.writeFile(input, this.painter.call<string>('prepare', { markdown, target: 'html', images: {} }));
    const body = await run(this.settings.pandoc, [input, '--from=markdown+mark', '--to=html5']);
    await fs.rm(tmp, { recursive: true, force: true });
    const html = this.painter.call<string>('compose', { title: file.basename, body, figures });
    const out = `${file.parent && file.parent.path !== '/' ? `${file.parent.path}/` : ''}${file.basename}.html`;
    await this.app.vault.adapter.write(out, html);
    new Notice(`Painter: ${out}`);
  }
}

class PainterSettings extends PluginSettingTab {
  plugin: PainterPlugin;

  constructor(app: App, plugin: PainterPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display() {
    const { containerEl } = this;
    containerEl.empty();
    const text = (name: string, desc: string, key: keyof Settings, placeholder = '') =>
      new Setting(containerEl)
        .setName(name)
        .setDesc(desc)
        .addText((t) =>
          t
            .setPlaceholder(placeholder)
            .setValue(String(this.plugin.settings[key]))
            .onChange(async (v) => {
              (this.plugin.settings[key] as string | number) = key === 'scale' ? Number(v) || DEFAULTS.scale : v.trim();
              await this.plugin.saveData(this.plugin.settings);
            }),
        );
    text('pandoc', 'The pandoc binary.', 'pandoc', 'pandoc');
    text('PDF engine', 'xelatex, lualatex, tectonic, typst, weasyprint…', 'pdfEngine', 'xelatex');
    text('Extra pandoc arguments', 'Space-separated, e.g. --toc --citeproc', 'pandocArgs');
    text('node', 'The node binary, to run archify.', 'node', 'node');
    text('archify script', 'Path to archify/bin/archify.mjs.', 'archifyScript', '/path/to/vendor/archify/archify/bin/archify.mjs');
    text('archify look', 'Optional CSS file over every archify figure (e.g. docs/archify/picasso.css).', 'archifyCss');
    text('Raster scale', 'Pixels per SVG unit for figures in the PDF (3 ≈ 300 dpi).', 'scale', '3');
  }
}
