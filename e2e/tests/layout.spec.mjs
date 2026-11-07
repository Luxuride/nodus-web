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

test('the rail reads Home, Feed, then the rest, with Compose before Settings', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')

  const labels = await railLabels(page)
  expect(labels.indexOf('Home')).toBeLessThan(labels.indexOf('Feed'))
  expect(labels.indexOf('Compose')).toBeLessThan(labels.indexOf('Settings'))
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

test('the composer opens as a panel and closes from its own button', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/')

  const panel = page.locator('section', { hasText: 'New post' })
  await expect(panel).toHaveCount(0)

  await page.getByRole('button', { name: 'Compose' }).click()
  await expect(panel).toBeVisible()

  await panel.getByRole('button', { name: 'Close' }).click()
  await expect(panel).toHaveCount(0)
})

test('the composer sits beside the page when there is room', async ({ page }) => {
  test.skip(page.viewportSize().width < 1280, 'a third column needs a wide window')

  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  await page.getByRole('button', { name: 'Compose' }).click()

  const panel = await page.locator('section', { hasText: 'New post' }).boundingBox()
  const main = await page.locator('main').boundingBox()
  const rail = await page.locator('nav').boundingBox()

  expect(panel.x).toBeGreaterThan(rail.x)
  expect(panel.x + panel.width).toBeLessThanOrEqual(main.x + 1)
})

test('the composer sits above the rail when the window is narrow', async ({ page }) => {
  test.skip(page.viewportSize().width >= 1280, 'a wide window gets the third column instead')

  await stubApi(page)
  await signedIn(page)
  await page.goto('/')
  await page.getByRole('button', { name: 'Compose' }).click()

  const panel = await page.locator('section', { hasText: 'New post' }).boundingBox()
  const rail = await page.locator('nav').boundingBox()
  const main = await page.locator('main').boundingBox()

  expect(panel.y).toBeLessThan(rail.y)
  expect(rail.y).toBeLessThan(main.y)
  expect(Math.round(panel.width)).toBe(Math.round(main.width))
})

test('the account menu lists the signed-in identity', async ({ page }) => {
  await stubApi(page)
  await signedIn(page)
  await page.goto('/settings')

  // The card, not the page: the screen's own `<section>` also contains the heading.
  const card = page.locator('section.card').filter({ hasText: 'Accounts' })
  await expect(card).toContainText('@ada')
  await expect(card).toContainText(HOST)
  await expect(card).toContainText('on screen')
  await expect(card).not.toContainText(ACCOUNT_ID)
})
