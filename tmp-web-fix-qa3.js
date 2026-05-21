const { chromium } = require("playwright");
(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 900, height: 700 } });
  const logs = [];
  page.on('console', msg => logs.push({type: msg.type(), text: msg.text()}));
  page.on('pageerror', err => logs.push({type: 'pageerror', text: String(err && (err.message || err))}));
  await page.goto('http://127.0.0.1:8084/', { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForTimeout(12000);
  const sample = await page.evaluate(() => {
    const canvas = document.querySelector('canvas');
    if (!canvas) return {hasCanvas:false};
    const gl = canvas.getContext('webgl2') || canvas.getContext('webgl');
    if (!gl) return {hasCanvas:true, hasGl:false};
    const pix = new Uint8Array(4);
    gl.readPixels(0,0,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pix);
    return {hasCanvas:true, hasGl:true, width:canvas.width, height:canvas.height, pixel:Array.from(pix), lost: !!(gl.isContextLost && gl.isContextLost())};
  });
  await page.screenshot({ path: 'tmp-web-fix-qa3.png' });
  console.log('SAMPLE', JSON.stringify(sample));
  console.log('LOGS', JSON.stringify(logs, null, 2));
  await browser.close();
})();
