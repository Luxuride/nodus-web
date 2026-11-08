//! Fixtures for the end-to-end tests.
//!
//! The client is the thing under test, so every response here is a constant: no
//! instance, no recommendation service, no database. That is what makes these tests
//! cheap enough to run on every commit — and honest, because a test that needed a live
//! instance would be measuring the instance.

export const HOST = 'stub.instance'
export const TOKEN = 'stub-token'
export const ACCOUNT_ID = '1'

export const account = {
  id: ACCOUNT_ID,
  acct: 'ada',
  display_name: 'Ada',
  note: '<p>Writes tests.</p>',
  avatar: 'https://files.example/ada.png',
  avatar_static: 'https://files.example/ada.png',
  header: 'https://files.example/header.png',
  bot: false,
  locked: false,
  followers_count: 3,
  following_count: 4,
  statuses_count: 5,
}

export const otherAccount = {
  ...account,
  id: '2',
  acct: 'grace',
  display_name: 'Grace',
}

/** A status, with the minimum an instance sends and the fields a view reads. */
export function status(overrides = {}) {
  return {
    id: '100',
    created_at: new Date('2026-11-07T12:00:00Z').toISOString(),
    content: '<p>Hello from the fixture.</p>',
    visibility: 'public',
    account,
    favourites_count: 1,
    reblogs_count: 2,
    replies_count: 0,
    favourited: false,
    reblogged: false,
    bookmarked: false,
    sensitive: false,
    spoiler_text: '',
    media_attachments: [],
    mentions: [],
    tags: [],
    ...overrides,
  }
}

/** The `localStorage` record the client writes at sign-in (`nodus-session/2`). */
export function sessionRecord({ host = HOST, token = TOKEN, id = ACCOUNT_ID, acct = 'ada' } = {}) {
  return ['nodus-session/2', host, token, id, acct].join('\n')
}

/**
 * Stub every request the client makes.
 *
 * One handler per shape rather than a generic echo: a generic echo lets a broken
 * request pass by matching the wrong route, which is the class of bug these tests
 * exist to catch.
 */
export async function stubApi(
  page,
  { statuses = [status()], status: single, notifications = [] } = {},
) {
  await page.route('**/api/v1/meta', (route) =>
    route.fulfill({
      json: { api: '1', contract: '0.1.0', min_contract: '0.1.0', features: [] },
    }),
  )
  await page.route('**/api/v1/accounts/verify_credentials', (route) =>
    route.fulfill({ json: account }),
  )
  await page.route('**/api/v1/timelines/home**', (route) => route.fulfill({ json: statuses }))
  await page.route('**/api/v1/timelines/public**', (route) => route.fulfill({ json: statuses }))
  await page.route(/\/api\/v1\/statuses\/[^/]+\/context$/, (route) =>
    route.fulfill({ json: { ancestors: [], descendants: [] } }),
  )
  await page.route(/\/api\/v1\/statuses\/[^/]+$/, (route) =>
    route.fulfill({ json: single ?? statuses[0] }),
  )
  await page.route('**/api/v1/notifications**', (route) =>
    route.fulfill({ json: notifications }),
  )
  await page.route('**/api/v1/me', (route) =>
    route.fulfill({
      json: {
        acct: 'ada',
        instance_host: HOST,
        sync_enabled: false,
        telemetry_opt_in: false,
        contract: '0.1.0',
      },
    }),
  )
  await page.route('**/api/v1/blocks**', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/v1/mutes**', (route) => route.fulfill({ json: [] }))
}

/** Seed the stored session so the client boots signed in. */
export async function signedIn(page, { host = HOST, id = ACCOUNT_ID, acct = 'ada' } = {}) {
  await page.addInitScript(
    ([record, instance, accountId, who]) => {
      localStorage.setItem('nodus.session', record)
      localStorage.setItem('nodus.host', instance)
      localStorage.setItem(
        'nodus.accounts/1',
        ['nodus-accounts/1', [instance, 'stub-token', accountId, who].join('\t')].join('\n'),
      )
      localStorage.setItem('nodus.account', `${instance}|${accountId}`)
    },
    [sessionRecord({ host, id, acct }), host, id, acct],
  )
}

/** One notification, shaped like the wire's. */
export function notification({ id = '5001', type = 'favourite' } = {}) {
  return {
    id,
    type,
    created_at: '2024-01-01T00:00:00.000Z',
    account,
    status: status(),
  }
}
