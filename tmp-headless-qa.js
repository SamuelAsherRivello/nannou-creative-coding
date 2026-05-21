const { chromium } = require("playwright");
(async () => {
  const port = 8097;
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
  const logs = [];
  page.on('console', m => logs.push({type:m.type(),text:m.text(),location:m.location()}));
  page.on('pageerror', e => logs.push({type:'pageerror',text:String(e.message || e),stack:e && e.stack}));
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForTimeout(15000);
    const probe = await page.evaluate(() => ({
      navigatorGPU: typeof navigator !== 'undefined' && !!navigator.gpu,
      webgl2: !!document.createElement('canvas').getContext('webgl2'),
      hasCanvas: !!document.querySelector('canvas'),
    }));
    const sample = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      if(!canvas) return { hasCanvas: false };
      const gl = canvas.getContext('webgl2') || canvas.getContext('webgl');
      const pix = new Uint8Array(4);
      if (gl) { gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pix); }
      return {
        hasCanvas: true,
        hasGl: !!gl,
        pixel: Array.from(pix)
      }
    });
    console.log('PROBE', JSON.stringify(probe));
    console.log('SAMPLE', JSON.stringify(sample));
    console.log('LOGS', JSON.stringify(logs, null, 2));
    await page.screenshot({ path: 'tmp-web-headless-check.png' });
  } finally {
    await browser.close();
    console.log('DONE');
  }
})();
