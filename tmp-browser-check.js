const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
  const logs = [];
  page.on('console', (msg) => {
    logs.push({ type: msg.type(), text: msg.text() });
  });
  page.on('pageerror', (err) => {
    logs.push({ type: 'pageerror', text: String(err && err.message || err) });
  });
  await page.goto('http://127.0.0.1:8123/', { waitUntil: 'networkidle', timeout: 20000 });
  await page.waitForTimeout(5000);

  const result = await page.evaluate(() => {
    const canvas = document.querySelector('canvas');
    if (!canvas) {
      return { ok: false, reason: 'no-canvas' };
    }
    const gl2 = canvas.getContext('webgl2');
    const gl = gl2 || canvas.getContext('webgl');
    if (!gl) {
      return { ok: false, reason: 'no-context' };
    }
    const lost = typeof gl.isContextLost === 'function' ? gl.isContextLost() : false;
    let sample = null;
    let err = null;
    try {
      const pix = new Uint8Array(4);
      gl.readPixels(Math.max(0, Math.floor((canvas.width || canvas.clientWidth) / 2), 1),
        Math.max(0, Math.floor((canvas.height || canvas.clientHeight) / 2), 1),
        1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pix);
      sample = Array.from(pix);
    } catch (e) {
      err = String(e && e.message ? e.message : e);
    }
    return {
      ok: true,
      hasCanvas: true,
      width: canvas.width,
      height: canvas.height,
      clientWidth: canvas.clientWidth,
      clientHeight: canvas.clientHeight,
      hasWebGl2: !!gl2,
      lost,
      sample,
      sampleErr: err,
      userAgent: navigator.userAgent,
    };
  });

  await page.screenshot({ path: 'tmp-web-smoke.png' });
  console.log('RESULT', JSON.stringify({ logs, result }));
  await browser.close();
})();
