//! Layout and navigation: the order of the rail, and where the composer goes.

import { expect, test } from '@playwright/test'

import { ACCOUNT_ID, HOST, signedIn, stubApi, hydrated } from '../fixtures.mjs'

/** The rail's labels, in the order a reader meets them. */
async function railLabels(page) {
  return page
    .locator('nav ul li')
    .allInnerTexts()
    .then((labels) => labels.map((label) => label.trim().split('\n')[0]).filter(Boolean))
}

test('the rail groups reading, finding and library, with the feeds under Feed', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  await hydrated(page)
  // The rail renders after the session is adopted, so read the labels only once it is
  // there: an empty list would make every `indexOf` -1 and the comparison meaningless.
  await expect(page.locator('nav a').first()).toBeVisible()

  const labels = await railLabels(page)
  // Home is the front door; the feeds are tabs inside it, not doors of their own.
  const order = [
    'Home', 'Messages', 'Notifications',
    'Search', 'People',
    'Saved', 'Scheduled', 'Lists',
    'Settings',
  ]
  const positions = order.map((label) => labels.indexOf(label))
  expect(positions).not.toContain(-1)
  // Ascending positions: the groups are in reading order, not insertion order.
  expect([...positions].sort((left, right) => left - right)).toEqual(positions)

  // The section titles are `hidden xl:block`, so on a narrow window they are present but
  // not visible; presence is what this test is about, the responsive rule is Tailwind's.
  await expect(page.locator('nav li.menu-title')).toHaveText(['Read', 'Find', 'Library', 'Account'])
})

test('a rail click is a client-side navigation, not a page load', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  await hydrated(page)
  await page.evaluate(() => {
    window.__marker = 'kept'
  })

  await page.getByRole('link', { name: 'Messages' }).click()

  await expect(page).toHaveURL(/\/messages$/)
  expect(await page.evaluate(() => window.__marker)).toBe('kept')
  await expect(page.locator('nav a.menu-active')).toContainText('Messages')
})

test('the composer sits above the home timeline, open', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  await hydrated(page)

  // Above the timeline, inside `main`, and already a form: no button to reveal it.
  const composer = page.locator('main textarea[aria-label="Write a post"]')
  await expect(composer).toBeVisible()
  await expect(page.getByRole('button', { name: 'Post', exact: true })).toBeVisible()

  // The status cards are `li.card`; the first one is the timeline's head.
  const timelineTop = await page.locator('main ul > li.card').first().boundingBox()
  const composerBox = await composer.boundingBox()
  expect(composerBox.y).toBeLessThan(timelineTop.y)
})

test('a feed can be hidden from the home tabs in settings', async ({ page }) => {
  test.skip(test.info().project.name !== 'desktop', 'the rail is hidden on a phone');
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  await hydrated(page)
  const tabs = page.getByRole('tablist').first()
  await expect(tabs.getByRole('tab', { name: 'Local' })).toBeVisible()

  await page.goto('/settings/nodus')
  await hydrated(page)
  await page.getByRole('checkbox', { name: /Local/ }).uncheck()

  // The tab is gone, and it is gone without a reload: the strip is rendered from the
  // preference signal.
  await page.goto('/')
  await hydrated(page)
  await expect(page.getByRole('tablist').first().getByRole('tab', { name: 'Local' })).toHaveCount(0)
  // Home is untouched.
  await expect(page.getByRole('tablist').first().getByRole('tab', { name: 'Home' })).toHaveCount(1)

  // Turning it back on restores it, so the test leaves the profile as it found it.
  await page.goto('/settings/nodus')
  await hydrated(page)
  await page.getByRole('checkbox', { name: /Local/ }).check()
})

test('the account menu lists the signed-in identity', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/settings')
  await hydrated(page)

  // The card by its heading: several cards mention accounts in their copy, so a text
  // filter matches more than the one under test.
  const card = page
    .locator('section.card')
    .filter({ has: page.getByRole('heading', { name: 'Accounts' }) })
  await expect(card).toContainText('@ada')
  await expect(card).toContainText(HOST)
  await expect(card).toContainText('on screen')
  await expect(card).not.toContainText(ACCOUNT_ID)
})

test('home stays in the navigation, and the feed can be closed', async ({ page }) => {
  test.skip(test.info().project.name !== 'desktop', 'the rail is hidden on a phone');
  await stubApi(page, {});
  await signedIn(page);
  await page.goto('/settings/nodus');
  await hydrated(page)
  // Home is the front door: its switch exists so the rule is visible, not so it can be
  // turned off.
  const home = page.locator('label', { hasText: 'Home' }).locator('input[type="checkbox"]').first();
  await expect(home).toBeDisabled();
  await expect(home).toBeChecked();

  const feed = page.locator('label', { hasText: 'Feed' }).locator('input[type="checkbox"]').first();
  await feed.uncheck();
  await page.goto('/local');
  await hydrated(page)
  await expect(page.getByRole('tablist').getByRole('tab', { name: 'Feed' })).toHaveCount(0);
  await expect(page.getByRole('tablist').getByRole('tab', { name: 'Home' })).toHaveCount(1);
});
