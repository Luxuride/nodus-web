import { test } from '@playwright/test';
import { stubApi } from '../fixtures.mjs';

const long = 'https://example.social/@someone/' + 'x'.repeat(160);

test('long unbreakable text', async ({ page }) => {
  await stubApi(page, { statuses: [{ id: '1', content: `<p>${long}</p>`, account: { id: '1', acct: 'a', display_name: 'A' } }] });
  await page.goto('/home');
  await page.waitForTimeout(500);
  console.log('long', await page.evaluate(() => ({ vw: document.documentElement.clientWidth, sw: document.documentElement.scrollWidth })));
});

test('composer schedule panel', async ({ page }) => {
  await stubApi(page, {});
  await page.goto('/home');
  await page.waitForTimeout(400);
  await page.locator('main textarea').first().fill('hello');
  await page.locator('button:has-text("▼")').first().click();
  await page.waitForTimeout(300);
  console.log('panel', await page.evaluate(() => ({ vw: document.documentElement.clientWidth, sw: document.documentElement.scrollWidth })));
});
