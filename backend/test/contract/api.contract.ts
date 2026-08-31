import { describe, expect, it } from 'vitest'

export interface ContractResponse {
  body: any
  response: {
    status: number
  }
}

export type ContractRequest = (path: string, init?: RequestInit) => Promise<ContractResponse>

export interface SeededCredentials {
  email: string
  password: string
}

export const SEEDED_ADMIN_CREDENTIALS: SeededCredentials = {
  email: 'admin@hesocial.com',
  password: 'admin123',
}

export interface ContractRunner {
  request: ContractRequest
  seededCredentials: SeededCredentials
  authImplemented?: boolean
  adminStatsExpectation?: 'authenticated' | 'not-implemented' | 'unauthorized'
  eventsImplemented?: boolean
}

export const defineContractTests = (runner: ContractRunner): void => {
  const postJson = (path: string, body: unknown) => runner.request(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  const authTest = it.skipIf(runner.authImplemented === false)
  // Express keeps its current behavior: the management routes are live there,
  // but the characterization temp-DB schema never loads the event-management
  // tables, so the express target leaves this flag unset (false) and skips
  // the block. The rust target flips it true.
  const eventTest = it.skipIf(runner.eventsImplemented !== true)

  describe('backend API characterization', () => {
    authTest('logs in a seeded administrator and rejects a wrong password', async () => {
      const valid = await postJson('/api/auth/login', runner.seededCredentials)
      expect(valid.response.status).toBe(200)
      expect(valid.body).toMatchObject({
        success: true,
        data: { token: expect.any(String) },
      })

      const invalid = await postJson('/api/auth/login', {
        email: runner.seededCredentials.email,
        password: 'not-the-password',
      })
      expect(invalid.response.status).toBe(401)
    })

    authTest('registers a new user', async () => {
      const result = await postJson('/api/auth/register', {
        email: 'phase-one@example.com',
        password: 'phase-one-password',
        firstName: 'Phase',
        lastName: 'One',
        age: 35,
        profession: 'Engineer',
        annualIncome: 5000000,
        netWorth: 30000000,
        bio: 'Characterization test account',
        interests: ['testing'],
      })

      expect(result.response.status).toBe(201)
      expect(result.body).toMatchObject({
        success: true,
        data: {
          token: expect.any(String),
          user: { email: 'phase-one@example.com' },
        },
      })
    })

    authTest('returns the current registration failure for a missing password', async () => {
      // Known bug, pinned deliberately: register must validate input (Kimi audit
      // finding 6) and return 4xx. Fixing the controller SHOULD turn this red —
      // then update this test to expect 400.
      const result = await postJson('/api/auth/register', {
        email: 'missing-password@example.com',
      })

      expect(result.response.status).toBe(500)
      expect(result.body).toMatchObject({
        success: false,
        error: 'Registration failed',
      })
    })

    authTest('validates a bearer token', async () => {
      const login = await postJson('/api/auth/login', runner.seededCredentials)
      const token = login.body.data.token as string

      const valid = await runner.request('/api/auth/validate', {
        headers: { authorization: `Bearer ${token}` },
      })
      expect(valid.response.status).toBe(200)
      expect(valid.body).toMatchObject({
        success: true,
        data: {
          user: { email: runner.seededCredentials.email },
          valid: true,
        },
      })

      const missing = await runner.request('/api/auth/validate')
      expect(missing.response.status).toBe(401)
      expect(missing.body).toMatchObject({ success: false })
    })

    it('returns the public events list shape', async () => {
      const result = await runner.request('/api/events')

      expect(result.response.status).toBe(200)
      expect(result.body).toMatchObject({
        success: true,
        data: expect.any(Array),
        pagination: {
          page: expect.any(Number),
          limit: expect.any(Number),
          total: expect.any(Number),
          totalPages: expect.any(Number),
        },
      })
    })

    it('protects database stats and permits an authenticated administrator', async () => {
      const unauthenticated = await runner.request('/api/admin/database/stats')

      if (runner.adminStatsExpectation === 'unauthorized') {
        // TODO(Phase 7): assert the authenticated-200 leg once the admin
        // database stats endpoint is ported. The auth middleware already guards
        // /api/admin/* ahead of the 501 fallback, so an unauthenticated caller
        // must be rejected before it ever reaches the fallback.
        expect([401, 403]).toContain(unauthenticated.response.status)
        expect(unauthenticated.body).toMatchObject({ success: false })
        return
      }

      if (runner.adminStatsExpectation === 'not-implemented') {
        // TODO(Phase 2): upgrade this contract to the shared 401/200 auth flow.
        expect(unauthenticated.response.status).toBe(501)
        expect(unauthenticated.body).toMatchObject({
          success: false,
          error: 'Endpoint not implemented yet',
        })
        return
      }

      expect([401, 403]).toContain(unauthenticated.response.status)

      const login = await postJson('/api/auth/login', runner.seededCredentials)
      const token = login.body.data.token as string
      const authenticated = await runner.request('/api/admin/database/stats', {
        headers: { authorization: `Bearer ${token}` },
      })

      expect(authenticated.response.status).toBe(200)
      expect(authenticated.body).toMatchObject({
        success: true,
        data: {
          tables: expect.any(Array),
        },
      })
    })
  })

  describe('event management (Phase 2c)', () => {
    const adminToken = async () => {
      const login = await postJson('/api/auth/login', runner.seededCredentials)
      expect(login.response.status).toBe(200)
      return login.body.data.token as string
    }
    const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` })
    const authedJson = (token: string) => ({
      authorization: `Bearer ${token}`,
      'content-type': 'application/json',
    })
    const eventFormPayload = (title: string) => ({
      title,
      description: `${title} description`,
      detailedDescription: `${title} long-form description`,
      categoryId: '1',
      venueId: '1',
      startDatetime: '2026-12-01T18:00:00.000Z',
      endDatetime: '2026-12-01T22:00:00.000Z',
      timezone: 'Asia/Taipei',
      capacityMin: 10,
      capacityMax: 40,
      pricePlatinum: 25000,
      priceDiamond: 20000,
      priceBlackCard: 15000,
      currency: 'TWD',
      requiredMembershipTiers: ['Diamond', 'Black Card'],
      requiredVerification: true,
      ageRestriction: { min: 25 },
      dressCode: 'Black Tie',
      language: 'Traditional Chinese',
      specialRequirements: '',
      inclusions: ['dinner'],
      exclusions: ['transport'],
      registrationOpensAt: '2026-10-01T00:00:00.000Z',
      registrationClosesAt: '2026-11-25T23:59:59.000Z',
      cancellationDeadline: '2026-11-28T23:59:59.000Z',
      waitlistEnabled: true,
      autoApproval: false,
      metaTitle: '',
      metaDescription: '',
      featuredImage: '',
      internalNotes: 'contract test event',
      costBreakdown: {},
      profitMargin: 0,
    })
    const createEvent = async (token: string, title: string) => {
      const created = await runner.request('/api/events', {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify(eventFormPayload(title)),
      })
      expect(created.response.status).toBe(201)
      return created.body.data.eventId as number
    }

    eventTest('returns the public event detail shape and 404s unknown ids', async () => {
      const detail = await runner.request('/api/events/2')
      expect(detail.response.status).toBe(200)
      expect(detail.body).toMatchObject({
        success: true,
        data: {
          id: 2,
          name: 'Autumn Yacht Social',
          dateTime: '2026-10-10T09:00:00.000Z',
          registrationDeadline: '2026-10-05T23:59:59.000Z',
          pricing: { vip: 18000, vvip: 18000, general: 18000, currency: 'TWD' },
          exclusivityLevel: null,
          dressCode: 'Resort Casual',
          capacity: 30,
          currentAttendees: 1,
          amenities: null,
          privacyGuarantees: null,
          videoUrl: null,
          venue: { id: 2, name: 'Keelung Luxury Yacht', city: 'Keelung' },
          category: { id: 2, name: '遊艇派對' },
          organizer: 'Admin User',
        },
      })
      // Raw management columns are admin-only (Phase 4 必辦).
      expect(detail.body.data.price_platinum).toBeUndefined()

      const missing = await runner.request('/api/events/424242')
      expect(missing.response.status).toBe(404)
      expect(missing.body).toMatchObject({ success: false, error: 'Event not found' })
    })

    eventTest('hides non-published events from anonymous callers but shows raw fields to admins', async () => {
      const anonymous = await runner.request('/api/events/1')
      expect(anonymous.response.status).toBe(404)
      expect(anonymous.body).toMatchObject({ success: false, error: 'Event not found' })

      const token = await adminToken()
      const detail = await runner.request('/api/events/1', { headers: authHeaders(token) })
      expect(detail.response.status).toBe(200)
      expect(detail.body.data).toMatchObject({
        status: 'pending_review',
        approval_status: 'pending',
        price_platinum: 12000,
        price_diamond: 12000,
        price_black_card: 12000,
        waitlist_enabled: true,
        registration_stats: {
          total_registrations: 0,
          confirmed_registrations: 0,
          waitlisted_registrations: 0,
          pending_registrations: 0,
        },
        waitlist_count: 0,
      })

      const seeded = await runner.request('/api/events/2', { headers: authHeaders(token) })
      expect(seeded.body.data.registration_stats).toMatchObject({
        total_registrations: 1,
        pending_registrations: 1,
      })
    })

    eventTest('rejects event creation without admin credentials', async () => {
      const anonymous = await runner.request('/api/events', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(eventFormPayload('Anon Gala')),
      })
      expect(anonymous.response.status).toBe(401)
      expect(anonymous.body).toMatchObject({
        success: false,
        error: 'Access token required',
      })

      const login = await postJson('/api/auth/login', {
        email: 'test.platinum@example.com',
        password: 'test123',
      })
      expect(login.response.status).toBe(200)
      const member = await runner.request('/api/events', {
        method: 'POST',
        headers: authedJson(login.body.data.token as string),
        body: JSON.stringify(eventFormPayload('Member Gala')),
      })
      expect(member.response.status).toBe(403)
      expect(member.body).toMatchObject({
        success: false,
        error: 'Admin access required',
      })
    })

    eventTest('creates an event from the EventForm payload shape', async () => {
      const token = await adminToken()
      const created = await runner.request('/api/events', {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify(eventFormPayload('Contract Gala')),
      })
      expect(created.response.status).toBe(201)
      expect(created.body).toEqual({
        success: true,
        message: 'Event created successfully',
        data: {
          eventId: expect.any(Number),
          slug: expect.stringMatching(/^contract-gala-\d+$/),
        },
      })

      const detail = await runner.request(
        `/api/events/${created.body.data.eventId}`,
        { headers: authHeaders(token) },
      )
      expect(detail.response.status).toBe(200)
      expect(detail.body.data).toMatchObject({
        title: 'Contract Gala',
        status: 'draft',
        approval_status: 'pending',
        category_id: 1,
        venue_id: 1,
        price_platinum: 25000,
        price_diamond: 20000,
        price_black_card: 15000,
        currency: 'TWD',
        capacity_min: 10,
        capacity_max: 40,
        required_membership_tiers: ['Diamond', 'Black Card'],
        required_verification: true,
        age_restriction: { min: 25 },
        inclusions: ['dinner'],
        exclusions: ['transport'],
        waitlist_enabled: true,
        auto_approval: false,
        language: 'Traditional Chinese',
        // Empty-string metaTitle falls back to the title (JS `||` semantics).
        meta_title: 'Contract Gala',
        meta_description: 'Contract Gala description',
      })
    })

    eventTest('updates an event through the camelCase whitelist', async () => {
      const token = await adminToken()
      const eventId = await createEvent(token, 'Update Gala')

      const updated = await runner.request(`/api/events/${eventId}`, {
        method: 'PUT',
        headers: authedJson(token),
        body: JSON.stringify({ pricePlatinum: 26500, dressCode: 'White Tie' }),
      })
      expect(updated.response.status).toBe(200)
      expect(updated.body).toEqual({ success: true, message: 'Event updated successfully' })

      const detail = await runner.request(`/api/events/${eventId}`, {
        headers: authHeaders(token),
      })
      expect(detail.body.data).toMatchObject({
        price_platinum: 26500,
        dress_code: 'White Tie',
      })

      const empty = await runner.request(`/api/events/${eventId}`, {
        method: 'PUT',
        headers: authedJson(token),
        body: JSON.stringify({ status: 'published' }),
      })
      expect(empty.response.status).toBe(400)
      expect(empty.body).toMatchObject({
        success: false,
        error: 'No valid fields to update',
      })

      const missing = await runner.request('/api/events/424242', {
        method: 'PUT',
        headers: authedJson(token),
        body: JSON.stringify({ title: 'nope' }),
      })
      expect(missing.response.status).toBe(404)
      expect(missing.body).toMatchObject({ success: false, error: 'Event not found' })
    })

    eventTest('drives the approve and publish transitions', async () => {
      const token = await adminToken()
      const eventId = await createEvent(token, 'Approval Gala')

      const earlyPublish = await runner.request(`/api/events/${eventId}/publish`, {
        method: 'POST',
        headers: authHeaders(token),
      })
      expect(earlyPublish.response.status).toBe(400)
      expect(earlyPublish.body).toMatchObject({
        success: false,
        error: 'Event not found or not approved for publishing',
      })

      const approved = await runner.request(`/api/events/${eventId}/approve`, {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ approved: true }),
      })
      expect(approved.response.status).toBe(200)
      expect(approved.body).toEqual({ success: true, message: 'Event approved successfully' })

      const published = await runner.request(`/api/events/${eventId}/publish`, {
        method: 'POST',
        headers: authHeaders(token),
      })
      expect(published.response.status).toBe(200)
      expect(published.body).toEqual({ success: true, message: 'Event published successfully' })

      const detail = await runner.request(`/api/events/${eventId}`, {
        headers: authHeaders(token),
      })
      expect(detail.body.data).toMatchObject({
        status: 'published',
        approval_status: 'approved',
        published_at: expect.any(String),
      })

      const rejectedId = await createEvent(token, 'Rejection Gala')
      const rejected = await runner.request(`/api/events/${rejectedId}/approve`, {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ approved: false }),
      })
      expect(rejected.response.status).toBe(200)
      expect(rejected.body).toEqual({ success: true, message: 'Event rejected successfully' })

      const missing = await runner.request('/api/events/424242/approve', {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ approved: true }),
      })
      expect(missing.response.status).toBe(404)
      expect(missing.body).toMatchObject({ success: false, error: 'Event not found' })
    })

    eventTest('enforces the delete rules', async () => {
      const token = await adminToken()

      // Seeded event 2 has a pending registration and cannot be deleted.
      const blocked = await runner.request('/api/events/2', {
        method: 'DELETE',
        headers: authHeaders(token),
      })
      expect(blocked.response.status).toBe(400)
      expect(blocked.body).toMatchObject({
        success: false,
        error: 'Cannot delete event with existing registrations. Archive the event instead.',
      })

      const eventId = await createEvent(token, 'Delete Gala')
      const deleted = await runner.request(`/api/events/${eventId}`, {
        method: 'DELETE',
        headers: authHeaders(token),
      })
      expect(deleted.response.status).toBe(200)
      expect(deleted.body).toEqual({ success: true, message: 'Event deleted successfully' })

      const missing = await runner.request(`/api/events/${eventId}`, {
        method: 'DELETE',
        headers: authHeaders(token),
      })
      expect(missing.response.status).toBe(404)
      expect(missing.body).toMatchObject({ success: false, error: 'Event not found' })
    })
  })
}
