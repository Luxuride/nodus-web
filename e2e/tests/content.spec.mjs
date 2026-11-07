//! Content: what a post shows, including what it deliberately hides.

import { expect, test } from '@playwright/test'

import { HOST, signedIn, status, stubApi } from '../fixtures.mjs'

test('the timeline renders the post the API returned', async ({ page }) => {
  await stubApi(page, { statuses: [status({ content: '<p>First fixture post.</p>' })] })
  await signedIn(page)
  await page.goto('/home')

  await expect(page.getByText('First fixture post.')).toBeVisible()
})

test('a sensitive post hides its media behind a label until asked', async ({ page }) => {
  const media = {
    id: 'm1',
    type: 'image',
    url: 'https://files.example/one.png',
    preview_url: 'https://files.example/one.png',
    description: 'a picture',
    meta_width: 800,
    meta_height: 600,
  }
  await stubApi(page, {
    statuses: [
      status({
        content: '<p>Behind a warning.</p>',
        sensitive: true,
        media_attachments: [media],
      }),
    ],
  })
  await signedIn(page)
  await page.goto('/home')

  const reveal = page.getByRole('button', { name: /sensitive/i })
  await expect(reveal).toBeVisible()
  await expect(page.locator('img[src*="one.png"]')).toHaveCount(0)

  await reveal.click()
  await expect(page.locator('img[src*="one.png"]')).toBeVisible()
})

test('a post link opened without a session names its instance', async ({ page }) => {
  await stubApi(page)
  await page.goto('/status/100')

  await expect(page.getByText('Read this post')).toBeVisible()
  await expect(page.getByRole('navigation')).toHaveCount(0)
})

test('a hosted post link renders the post for an anonymous reader', async ({ page }) => {
  await stubApi(page, { statuses: [status({ content: '<p>Public and readable.</p>' })] })
  await page.goto(`/status/${HOST}/100`)

  await expect(page.getByText('Public and readable.')).toBeVisible()
  await expect(page.getByText(/without signing in/)).toBeVisible()
  await expect(page.getByRole('navigation')).toHaveCount(0)
})
