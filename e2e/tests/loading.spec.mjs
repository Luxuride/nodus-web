//! A second visit to a timeline must not go back to the network.
//!
//! The SDK holds the page and its timestamp, so navigating away and back within the TTL is
//! a read of state that is already in the wasm module. This test is the guard on that
//! promise: it counts the requests the browser actually made, not the ones the UI implies.

import { expect, test } from '@playwright/test'

import { signedIn, stubApi } from '../fixtures.mjs'

test('a timeline is fetched once, and revisited from the store', async ({ page }) => {
  test.skip(test.info().project.name !== 'desktop', 'the phone shell renders the same store from a different bar');
  const asked = []
  page.on('request', (request) => {
    if (request.url().includes('/api/v1/timelines/')) {
      asked.push(new URL(request.url()).pathname + new URL(request.url()).search)
    }
  })

  await stubApi(page)
  await signedIn(page)
  await page.goto('/home')
  await expect(page.getByText('Hello from the fixture.')).toBeVisible()

  // Away and back: the same page, inside the minute it stays fresh.
  const tabs = page.getByRole('tablist').first()
  await tabs.getByRole('tab', { name: 'Local' }).click()
  await tabs.getByRole('tab', { name: 'Home' }).click()
  await expect(page.getByText('Hello from the fixture.')).toBeVisible()

  const home = asked.filter((path) => path.includes('/timelines/home'))
  expect(home, `requests: ${asked.join(', ')}`).toHaveLength(1)
})
