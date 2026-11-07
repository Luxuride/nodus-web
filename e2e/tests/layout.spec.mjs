//! Layout and navigation: the order of the rail, and where the composer goes.

import { expect, test } from '@playwright/test'

import { ACCOUNT_ID, HOST, signedIn, stubApi } from '../fixtures.mjs'

/** The rail's labels, in the order a reader meets them. */
async function railLabels(page) {
  return page
    .locator('nav ul li')
    .allInnerTexts()
    .then((labels) => labels.map((label) => label.trim().split('\n')[0]).filter(Boolean))
}

test('the rail groups reading, finding and library, with Compose before Settings', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  // The rail renders after the session is adopted, so read the labels only once it is
  // there: an empty list would make every `indexOf` -1 and the comparison meaningless.
  await expect(page.getByRole('link', { name: 'Home' })).toBeVisible()

  const labels = await railLabels(page)
  const order = [
    'Compose',
    'Feed', 'Home', 'Messages', 'Notifications',
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
  await page.evaluate(() => {
    window.__marker = 'kept'
  })

  await page.getByRole('link', { name: 'Feed' }).click()

  await expect(page).toHaveURL(/\/$/)
  expect(await page.evaluate(() => window.__marker)).toBe('kept')
  await expect(page.locator('nav a.menu-active')).toHaveText('Feed')
})

test('Compose is a link at the top of the rail, not a panel', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')

  const compose = page.locator('nav a[href="/compose"]')
  await expect(compose).toBeVisible()

  // First item in the rail: writing is a destination, not a corner of the timeline that
  // can be left open over it.
  const first = page.locator('nav li').first()
  await expect(first).toContainText('Compose')

  await page.evaluate(() => {
    window.__marker = 'kept'
  })
  await compose.click()
  await expect(page).toHaveURL(/\/compose$/)
  expect(await page.evaluate(() => window.__marker)).toBe('kept')
})

test('the account menu lists the signed-in identity', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/settings')

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
