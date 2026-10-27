// One file, the way Obsidian loads plugins: main.js with the WebAssembly core inside.
// `npm run build` writes main.js here; copy main.js, manifest.json and styles.css to
// <vault>/.obsidian/plugins/obsidian-painter/ (the release zip holds exactly those).
import esbuild from 'esbuild';

await esbuild.build({
  entryPoints: ['src/main.ts'],
  bundle: true,
  format: 'cjs',
  platform: 'node',
  target: 'es2020',
  external: ['obsidian', 'electron'],
  loader: { '.wasm': 'binary' },
  outfile: 'main.js',
  minify: true,
  logLevel: 'info',
});
