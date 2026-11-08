import { chromium } from 'playwright'
const b = await chromium.launch(); const p = await b.newPage()
p.on('console', m => console.log('LOG', m.type(), m.text().slice(0,160)))
p.on('pageerror', e => console.log('ERR', e.message.slice(0,200)))
await p.goto('http://127.0.0.1:4173/', {waitUntil:'load'})
await p.waitForTimeout(3000)
console.log('nodus=', await p.evaluate(() => window.__nodus))
console.log('wasm=', await p.evaluate(() => document.querySelectorAll('script[type=module]').length))
await b.close()
