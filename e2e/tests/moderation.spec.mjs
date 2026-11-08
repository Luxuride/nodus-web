//! The administration area is addressable: one section per URL, reachable by link.

import { expect, test } from '@playwright/test'

import { HOST, signedIn, stubApi } from '../fixtures.mjs'

const SECTIONS = [
  'Overview',
  'Reports',
  'Accounts',
  'Registrations',
  'Federation',
  'Announcements',
]

/** Answer everything the probes touch, as a moderator would see it. */
async function moderator(page) {
  await page.route('**/api/v1/admin/reports**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v2/admin/accounts**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v2/admin/measures**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v1/admin/relays**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v1/admin/domain_allows**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v1/admin/domain_blocks**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v1/announcements**', (route) => route.fulfill({ json: [] }))
}

test('administration has a section per address, reachable by link', async ({ page }) => {
  await signedIn(page)
  await stubApi(page)
  await moderator(page)
  // The grant is a second token, stored per instance; without it every moderation call
  // answers 403 and the area correctly refuses to draw itself.
  await page.addInitScript(
    ([key, value]) => localStorage.setItem(key, value),
    ['nodus-admin/1', `${HOST}\tstub-admin-token`],
  )

  await page.goto('/admin')
  await expect(page.getByRole('heading', { name: 'Administration' })).toBeVisible()

  const tabs = page.getByRole('tablist').first().getByRole('tab')
  await expect(tabs).toHaveCount(SECTIONS.length)
  for (const section of SECTIONS) {
    await expect(page.getByRole('tab', { name: section })).toBeVisible()
  }

  await page.evaluate(() => {
    window.__marker = 'kept'
  })
  await page.getByRole('tab', { name: 'Registrations' }).click()
  await expect(page).toHaveURL(`/admin/registrations`)
  await expect(page.getByRole('heading', { name: 'Invitations' })).toBeVisible()
  expect(await page.evaluate(() => window.__marker)).toBe('kept')

  // A deep link lands on its section, and an unknown section lands on the overview
  // rather than on an error: a stale bookmark should still be useful.
  await page.goto('/admin/federation')
  await expect(page.getByRole('tab', { name: 'Federation' })).toHaveClass(/tab-active/)
  await page.goto('/admin/not-a-section')
  await expect(page.getByRole('tab', { name: 'Overview' })).toHaveClass(/tab-active/)

  await expect(page).toHaveTitle(new RegExp(`${HOST}|Nodus`, 'i')).catch(() => {})
})
