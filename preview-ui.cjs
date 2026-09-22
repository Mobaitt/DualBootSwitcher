const { chromium } = require('C:/Users/mobai/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const fs = require('fs');
const path = require('path');
(async () => {
  const browser = await chromium.launch({headless: true, channel:'msedge'});
  const page = await browser.newPage({viewport:{width:800,height:430}, deviceScaleFactor:1});
  await page.route('http://preview/**', route => {
    const pathname = new URL(route.request().url()).pathname;
    const file = path.join(__dirname, 'dist', pathname === '/' ? 'index.html' : pathname);
    route.fulfill({body:fs.readFileSync(file), contentType:file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : 'text/html'});
  });
  await page.addInitScript(() => {
    localStorage.setItem('dualboot-theme','dark');
    window.__TAURI_INTERNALS__ = {transformCallback:()=>1, unregisterCallback:()=>{}, invoke:async (cmd)=> cmd.includes('boot_info') ? { currentOs:'Windows', uefi:true, bootNext:'ubuntu', bootOrder:['windows','ubuntu'], entries:[{id:'windows',name:'Windows Boot Manager',entryType:'windows',path:'\\EFI\\Microsoft\\Boot\\bootmgfw.efi',order:0},{id:'ubuntu',name:'ubuntu',entryType:'linux',path:'\\EFI\\ubuntu\\shimx64.efi',next:true,order:1}]} : 1};
  });
  await page.goto('http://preview/');
  await page.getByText('Windows Boot Manager').first().waitFor();
  await page.screenshot({path:'ui-preview-dark.png'});
  console.log(await page.locator('.entry-card').evaluateAll(rows=>rows.map(r=>({bottom:r.getBoundingClientRect().bottom,height:r.getBoundingClientRect().height}))));
  await browser.close();
})();
