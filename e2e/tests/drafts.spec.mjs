//! Drafts: text that was never sent, and the `:draft` list that brings it back.
//!
//! The point of a draft is that it survives the document, so the test throws the page away
//! the way a writer would — write, reload, resume — and checks that the pickers come back
//! with the text rather than resetting to their defaults.

import { expect, test } from '@playwright/test'

import { signedIn, status, stubApi } from '../fixtures.mjs'

/** Writes into the ProseMirror body and waits for the word count to catch up. */
async function write(page, text) {
  const body = page.locator('main .ProseMirror')
  await body.click()
  await page.keyboard.type(text)
  await expect(body).toContainText(text)
}

test('unsent text comes back through :draft, with its visibility', async ({ page }) => {
  await stubApi(page, { statuses: [status()] })
  await signedIn(page)
  await page.goto('/')

  await write(page, 'Half a thought about relays')

  // The document goes away; the draft is written on the way out.
  await page.reload()
  await expect(page.locator('main .ProseMirror')).toHaveText('')

  await write(page, ':draft')

  const draft = page.getByRole('option').filter({ hasText: 'Half a thought about relays' })
  await expect(draft).toBeVisible()
  await draft.click()

  await expect(page.locator('main .ProseMirror')).toContainText('Half a thought about relays')
  // Resuming takes the draft out of the list: a draft that stays put can be sent twice.
  await expect(page.getByRole('option')).toHaveCount(0)
})

test('posting clears the draft rather than offering it again', async ({ page }) => {
  await stubApi(page, { statuses: [status()] })
  await signedIn(page)
  await page.goto('/')

  await write(page, 'Sent once')
  await page.reload()
  await write(page, ':draft')
  await page.getByRole('option').filter({ hasText: 'Sent once' }).click()

  await page.getByRole('button', { name: 'Post', exact: true }).click()

  // The composer empties and the confirmation names what actually went out.
  await expect(page.locator('main .ProseMirror')).toHaveText('')
  await expect(page.getByRole('status')).toContainText('Post published as')

  await write(page, ':draft')
  await expect(page.getByRole('option')).toHaveCount(0)
})
