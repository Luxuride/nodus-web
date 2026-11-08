import { test } from '@playwright/test';
import { stubApi } from '../fixtures.mjs';

const ROUTES = ['/home', '/local', '/federated', '/feed', '/notifications', '/search', '/people', '/saved/favourites', '/scheduled', '/lists', '/settings/account', '/settings/privacy', '/messages'];

for (const route of ROUTES) {
  test(`no sideways scroll on ${route}`, async ({ page }) => {
    await stubApi(page, {});
    await page.goto(route);
    await page.waitForTimeout(600);
    const report = await page.evaluate(() => {
      const vw = document.documentElement.clientWidth;
      const wide = [];
      for (const el of document.querySelectorAll('*')) {
        const box = el.getBoundingClientRect();
        if (box.width === 0 || box.height === 0) continue;
        if (box.right > vw + 1 || box.left < -1) {
          wide.push(`${el.tagName.toLowerCase()}.${String(el.className).slice(0, 50)} [${Math.round(box.left)}..${Math.round(box.right)}]`);
        }
      }
      return { vw, scrollWidth: document.documentElement.scrollWidth, wide: wide.slice(0, 6) };
    });
    console.log(route, JSON.stringify(report));
    if (report.scrollWidth > report.vw + 1) console.log('WIDE:', report.wide.join(' | '));
  });
}
