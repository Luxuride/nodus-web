//! Inline `:code` completion: instance emoji first, standard emoji as the baseline.

import { expect, test } from '@playwright/test'

import { hydrated, signedIn, status, stubApi } from '../fixtures.mjs'

test('a partial code lists the standard emoji, and Enter takes it', async ({ page }) => {
  await stubApi(page, { statuses: [status()] })
  await signedIn(page)
  await page.goto('/')
  await hydrated(page)

  const body = page.locator('#nodus-composer-body')
  await body.click()
  await page.keyboard.type(':smi')

  const hints = page.locator('[data-emoji-hints] [role="option"]')
  await expect(hints.first()).toBeVisible()
  await expect(hints.first()).toContainText(':smile:')

  await page.keyboard.press('Enter')
  // The character, not an atom: a standard emoji is text on every other server too.
  await expect(page.locator('.ProseMirror')).toContainText('😄')
  await expect(hints).toHaveCount(0)
})
