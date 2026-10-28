// The WebAssembly core through the same glue the plugin uses (src/painter.ts).
// Needs `npm run wasm` first; node strips the TypeScript types itself.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { Painter } from '../src/painter.ts';

const painter = await Painter.load(readFileSync(new URL('../build/painter.wasm', import.meta.url)));

test('draws the README drawing from its Desmos state', () => {
  const state = readFileSync(new URL('../../docs/drawings/pirsig.desmos.json', import.meta.url), 'utf8');
  const r = painter.call('desmos', { source: state });
  assert.deepEqual(r.skipped, []);
  assert.equal((r.svg.match(/<path|<circle/g) ?? []).length, 130);
});

test('finds figures and prepares a note for pandoc', () => {
  const markdown = 'See [[a|b]].\n```desmos\ny=x\n```\n';
  const [block] = painter.call('blocks', { markdown });
  assert.equal(block.lang, 'desmos');
  assert.equal(block.id, painter.call('id', { lang: 'desmos', source: 'y=x' }));
  const md = painter.call('prepare', { markdown, target: 'pdf', images: { [block.id]: '/t/f.png' } });
  assert.match(md, /See b\./);
  assert.match(md, /!\[\]\(\/t\/f\.png\)\{width=100%\}/);
});

test('composes one page with a shadow root per figure', () => {
  const html = painter.call('compose', { title: 'T', body: '<div data-painter="x"></div>', figures: { x: { css: '.a{}', svg: '<svg></svg>' } } });
  assert.match(html, /<template shadowrootmode="open">/);
});

test('reports errors as exceptions', () => {
  assert.throws(() => painter.call('nope', {}), /unknown op/);
  assert.throws(() => painter.call('archify', { page: '<p>no svg</p>' }), /without an SVG/);
});

test('survives many calls (memory is freed)', () => {
  for (let i = 0; i < 2000; i++) painter.call('id', { lang: 'desmos', source: `y=${i}x` });
});
