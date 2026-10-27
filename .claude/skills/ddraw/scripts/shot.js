// Renders an SVG file to a PNG with the pre-installed Chromium, so you can look at it.
//
//   NODE_PATH=$(npm root -g) node .claude/skills/ddraw/scripts/shot.js in.svg out.png [scale]
//
// scale defaults to 3. Look at the PNG at scale 3 (detail) and at scale 1 (the README size).
const { chromium } = require('playwright');
const fs = require('fs');

(async () => {
  const [inp, out, scale] = process.argv.slice(2);
  if (!inp || !out) {
    console.error('usage: node shot.js in.svg out.png [scale]');
    process.exit(2);
  }
  const svg = fs.readFileSync(inp, 'utf8');
  const browser = await chromium.launch();
  const page = await browser.newPage({ deviceScaleFactor: Number(scale || 3) });
  await page.setContent(`<body style="margin:0;background:#fff">${svg}</body>`);
  await page.locator('svg').screenshot({ path: out });
  await browser.close();
})();
