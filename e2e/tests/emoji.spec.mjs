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
  // Several codes start with `smi` (`:smiley:` and `:smile:` both do), so the assertion is
  // about the completion working, not about which of the two ranks first.
  await expect(hints.first()).toContainText(':smi')

  await page.keyboard.press('Enter')
  // The character, not an atom: a standard emoji is text on every other server too.
  const written = page.locator('.ProseMirror')
  await expect(written).not.toContainText(':smi')
  await expect(written).toContainText(/\p{Extended_Pictographic}/u)
  await expect(hints).toHaveCount(0)
})
