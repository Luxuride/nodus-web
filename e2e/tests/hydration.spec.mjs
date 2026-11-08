//! A cold visit must fill itself.
//!
//! The page store notifies subscribers when it changes; a screen that reads it but is not
//! subscribed fetches its data and then goes on rendering the empty state until it is
//! remounted. That is invisible in a screenshot and obvious in a test like this one.

import { expect, test } from '@playwright/test'

import { notification, signedIn, stubApi } from '../fixtures.mjs'

test('the first visit to notifications shows what was fetched', async ({ page }) => {
  await stubApi(page, { notifications: [notification()] })
  await signedIn(page)
  await page.goto('/notifications')

  // The notification carries a status, whose body is the fixture's text.
  await expect(page.getByText('Hello from the fixture.')).toBeVisible()
  await expect(page.getByText('Nothing here yet.')).toHaveCount(0)
})
