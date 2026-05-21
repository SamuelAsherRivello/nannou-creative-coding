const { chromium } = require("playwright");
(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
  const logs = [];
  page.on('console', msg => logs.push({type: msg.type(), text: msg.text(), location: msg.location()}));
  page.on('pageerror', err => logs.push({type: 'pageerror', text: String(err && (err.message || err)), stack: err && err.stack}));
  page.on('requestfailed', req => logs.push({type: 'requestfailed', text: req.url()}));
  try {
    await page.goto('http://127.0.0.1:8083/', { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(12000);
    const sample = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      if (!canvas) return { hasCanvas: false };
      const gl = canvas.getContext('webgl2');
      if (!gl) return { hasCanvas: true, hasGl: false };
      const pix = new Uint8Array(4);
      gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pix);
      const offscreen = new ImageData(new Uint8ClampedArray(canvas.width * canvas.height * 4), canvas.width, canvas.height);
      return {
        hasCanvas: true,
        hasGl: true,
        width: canvas.width,
        height: canvas.height,
        pixel: Array.from(pix),
        lost: !!(gl.isContextLost && gl.isContextLost()),
        style: {
          width: window.getComputedStyle(canvas).width,
          height: window.getComputedStyle(canvas).height,
        },
      };
    });
    await page.screenshot({ path: 'tmp-web-fix-qa.png' });
    console.log('SAMPLE', JSON.stringify(sample));
    console.log('LOGS', JSON.stringify(logs, null, 2));
  } finally {
    await browser.close();
  }
})();
