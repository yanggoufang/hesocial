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

// Plain `user`-role account seeded on both targets (D1 seed.sql and Express
// ensureSeedUsers) — used to pin the 403 guard responses.
export const SEEDED_PLATINUM_CREDENTIALS: SeededCredentials = {
  email: 'test.platinum@example.com',
  password: 'test123',
}

export interface ContractRunner {
  request: ContractRequest
  seededCredentials: SeededCredentials
  authImplemented?: boolean
  adminStatsExpectation?: 'authenticated' | 'not-implemented' | 'unauthorized'
  adminImplemented?: boolean
  // Rust-only: Express's `GET /api/users` always 500s (DuckDB COUNT(*) comes
  // back as BigInt and `Math.ceil(total / limit)` throws "Cannot mix BigInt
  // and other types"), and `stats/overview` always 500s (DuckDB's date()
  // takes one argument, not the SQLite two-argument spelling). The Rust port
  // implements the intended behavior, so list/stats assertions run there
  // only — same split as the participants contract.
  adminListImplemented?: boolean
  eventsImplemented?: boolean
  registrationsImplemented?: boolean
  participantsImplemented?: boolean
  salesImplemented?: boolean
  salesFlowImplemented?: boolean
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

      // Known live drift: registrationRoutes.ts queries the nonexistent
      // `event_registrations` table (the real table is `registrations`). Both
      // targets deliberately pin the resulting 500 until that API is changed
      // as a separately approved compatibility fix.
      const registrationStats = await runner.request('/api/registrations/stats/2')
      expect(registrationStats.response.status).toBe(500)
      expect(registrationStats.body).toMatchObject({
        success: false,
        error: expect.any(String),
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

  describe('user management (Phase 7)', () => {
    const adminTest = it.skipIf(runner.adminImplemented !== true)
    const adminListTest = it.skipIf(runner.adminListImplemented !== true)
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

    adminTest('rejects anonymous and non-admin callers', async () => {
      const anonymous = await runner.request('/api/users')
      expect(anonymous.response.status).toBe(401)
      expect(anonymous.body).toMatchObject({ success: false })

      const login = await postJson('/api/auth/login', SEEDED_PLATINUM_CREDENTIALS)
      expect(login.response.status).toBe(200)
      const token = login.body.data.token as string

      const list = await runner.request('/api/users', { headers: authHeaders(token) })
      expect(list.response.status).toBe(403)
      expect(list.body).toMatchObject({ success: false, error: 'Admin access required' })

      const roleChange = await runner.request('/api/users/any-id/role', {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ role: 'admin' }),
      })
      expect(roleChange.response.status).toBe(403)
      expect(roleChange.body).toMatchObject({
        success: false,
        error: 'Super admin access required',
      })
    })

    adminListTest('lists users with pagination, filters, and the raw row shape', async () => {
      const token = await adminToken()

      const list = await runner.request('/api/users', { headers: authHeaders(token) })
      expect(list.response.status).toBe(200)
      expect(list.body).toMatchObject({
        success: true,
        pagination: { page: 1, limit: 20 },
      })
      expect(Array.isArray(list.body.data)).toBe(true)
      expect(list.body.pagination.total).toBeGreaterThanOrEqual(2)

      const adminRow = list.body.data.find(
        (user: any) => user.email === runner.seededCredentials.email,
      )
      expect(adminRow).toMatchObject({
        role: 'super_admin',
        membership_tier: 'Black Card',
        verification_status: 'approved',
        is_verified: true,
      })
      expect(Array.isArray(adminRow.interests)).toBe(true)

      const byRole = await runner.request('/api/users?role=super_admin', {
        headers: authHeaders(token),
      })
      expect(byRole.response.status).toBe(200)
      expect(byRole.body.data.length).toBeGreaterThanOrEqual(1)
      for (const user of byRole.body.data) {
        expect(user.role).toBe('super_admin')
      }

      const bySearch = await runner.request('/api/users?search=hesocial', {
        headers: authHeaders(token),
      })
      expect(bySearch.response.status).toBe(200)
      expect(bySearch.body.data.map((user: any) => user.email)).toContain(
        runner.seededCredentials.email,
      )

      const byVerification = await runner.request('/api/users?verificationStatus=approved', {
        headers: authHeaders(token),
      })
      expect(byVerification.response.status).toBe(200)
      for (const user of byVerification.body.data) {
        expect(user.verification_status).toBe('approved')
      }
    })

    adminListTest('returns the user statistics overview', async () => {
      const token = await adminToken()

      const stats = await runner.request('/api/users/stats/overview', {
        headers: authHeaders(token),
      })
      expect(stats.response.status).toBe(200)
      expect(stats.body).toMatchObject({
        success: true,
        data: {
          totalUsers: expect.any(Number),
          usersByRole: expect.any(Array),
          usersByMembershipTier: expect.any(Array),
          usersByVerificationStatus: expect.any(Array),
          recentRegistrations: expect.any(Number),
        },
      })
      expect(stats.body.data.totalUsers).toBeGreaterThanOrEqual(2)
    })

    adminTest('supports the full fetch/update/verify/role/delete lifecycle', async () => {
      const token = await adminToken()

      const email = `admin-contract-${Date.now()}@example.com`
      const registered = await postJson('/api/auth/register', {
        email,
        password: 'contract-password-123',
        firstName: 'Contract',
        lastName: 'Target',
        age: 33,
        profession: 'Auditor',
        annualIncome: 6000000,
        netWorth: 31000000,
        bio: 'lifecycle fixture',
        interests: ['testing'],
      })
      expect(registered.response.status).toBe(201)
      const userId = String(registered.body.data.user.id)

      const fetched = await runner.request(`/api/users/${userId}`, {
        headers: authHeaders(token),
      })
      expect(fetched.response.status).toBe(200)
      expect(fetched.body).toMatchObject({
        success: true,
        data: { email, first_name: 'Contract' },
      })

      const updated = await runner.request(`/api/users/${userId}`, {
        method: 'PUT',
        headers: authedJson(token),
        body: JSON.stringify({ firstName: 'Updated', interests: ['contracts'] }),
      })
      expect(updated.response.status).toBe(200)
      expect(updated.body).toMatchObject({
        success: true,
        message: 'User updated successfully',
      })

      const refetched = await runner.request(`/api/users/${userId}`, {
        headers: authHeaders(token),
      })
      expect(refetched.body.data.first_name).toBe('Updated')
      expect(refetched.body.data.interests).toEqual(['contracts'])

      const empty = await runner.request(`/api/users/${userId}`, {
        method: 'PUT',
        headers: authedJson(token),
        body: JSON.stringify({ unknownKey: 1 }),
      })
      expect(empty.response.status).toBe(400)
      expect(empty.body).toMatchObject({
        success: false,
        error: 'No valid fields to update',
      })

      const badVerify = await runner.request(`/api/users/${userId}/verify`, {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ status: 'maybe' }),
      })
      expect(badVerify.response.status).toBe(400)
      expect(badVerify.body).toMatchObject({
        success: false,
        error: 'Invalid verification status',
      })

      const verified = await runner.request(`/api/users/${userId}/verify`, {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ status: 'approved' }),
      })
      expect(verified.response.status).toBe(200)
      expect(verified.body).toMatchObject({
        success: true,
        message: 'User verified successfully',
      })

      const afterVerify = await runner.request(`/api/users/${userId}`, {
        headers: authHeaders(token),
      })
      expect(afterVerify.body.data).toMatchObject({
        verification_status: 'approved',
        is_verified: true,
      })

      const badRole = await runner.request(`/api/users/${userId}/role`, {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ role: 'manager' }),
      })
      expect(badRole.response.status).toBe(400)
      expect(badRole.body).toMatchObject({ success: false, error: 'Invalid role' })

      const promoted = await runner.request(`/api/users/${userId}/role`, {
        method: 'POST',
        headers: authedJson(token),
        body: JSON.stringify({ role: 'admin' }),
      })
      expect(promoted.response.status).toBe(200)
      expect(promoted.body).toMatchObject({
        success: true,
        message: 'User role updated successfully',
      })

      const missing = await runner.request(
        '/api/users/00000000-0000-4000-8000-000000000000',
        { headers: authHeaders(token) },
      )
      expect(missing.response.status).toBe(404)
      expect(missing.body).toMatchObject({ success: false, error: 'User not found' })

      const deleted = await runner.request(`/api/users/${userId}`, {
        method: 'DELETE',
        headers: authHeaders(token),
      })
      expect(deleted.response.status).toBe(200)
      expect(deleted.body).toMatchObject({
        success: true,
        message: 'User deleted successfully',
      })

      const gone = await runner.request(`/api/users/${userId}`, {
        headers: authHeaders(token),
      })
      expect(gone.response.status).toBe(404)
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

  if (runner.registrationsImplemented === true) {
    describe('registrations and waitlist (Phase 2d)', () => {
      const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` })
      const authedJson = (token: string) => ({
        authorization: `Bearer ${token}`,
        'content-type': 'application/json',
      })
      const tokenFor = async (credentials: SeededCredentials) => {
        const login = await postJson('/api/auth/login', credentials)
        expect(login.response.status).toBe(200)
        return login.body.data.token as string
      }
      const eventPayload = (title: string, capacityMax: number, waitlistEnabled = true) => ({
        title,
        description: `${title} description`,
        detailedDescription: `${title} long-form description`,
        categoryId: '1',
        venueId: '1',
        startDatetime: '2026-12-20T18:00:00.000Z',
        endDatetime: '2026-12-20T22:00:00.000Z',
        timezone: 'Asia/Taipei',
        capacityMin: 1,
        capacityMax,
        pricePlatinum: 1000,
        priceDiamond: 900,
        priceBlackCard: 800,
        currency: 'TWD',
        requiredMembershipTiers: ['Platinum', 'Diamond', 'Black Card'],
        requiredVerification: true,
        dressCode: 'Smart Casual',
        language: 'Traditional Chinese',
        inclusions: [],
        exclusions: [],
        registrationOpensAt: '2026-08-01T00:00:00.000Z',
        registrationClosesAt: '2026-12-15T23:59:59.000Z',
        cancellationDeadline: '2026-12-18T23:59:59.000Z',
        waitlistEnabled,
        autoApproval: false,
      })
      const createPublishedEvent = async (
        token: string,
        title: string,
        capacityMax: number,
        waitlistEnabled = true,
      ) => {
        const created = await runner.request('/api/events', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify(eventPayload(title, capacityMax, waitlistEnabled)),
        })
        expect(created.response.status).toBe(201)
        const eventId = created.body.data.eventId as number
        const approved = await runner.request(`/api/events/${eventId}/approve`, {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify({ approved: true }),
        })
        expect(approved.response.status).toBe(200)
        const published = await runner.request(`/api/events/${eventId}/publish`, {
          method: 'POST',
          headers: authHeaders(token),
        })
        expect(published.response.status).toBe(200)
        return eventId
      }
      const register = (token: string, eventId: number, specialRequests?: string) =>
        runner.request(`/api/registrations/events/${eventId}`, {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify({ specialRequests }),
        })
      const eventDetail = (token: string, eventId: number) =>
        runner.request(`/api/events/${eventId}`, { headers: authHeaders(token) })

      it('rejects an unauthenticated registration before touching D1', async () => {
        const result = await runner.request('/api/registrations/events/2', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: '{}',
        })
        expect(result.response.status).toBe(401)
        expect(result.body).toEqual({ success: false, error: 'Access token required' })
      })

      it('registers atomically, exposes the UI fields, and rejects a duplicate', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const eventId = await createPublishedEvent(token, 'Registration Contract Gala', 4)
        const created = await register(token, eventId, 'Window seat')

        expect(created.response.status).toBe(201)
        expect(created.body).toEqual({
          success: true,
          data: {
            registrationId: expect.any(Number),
            status: 'pending',
            message: 'Registration submitted successfully. Pending approval.',
          },
        })

        const detail = await eventDetail(token, eventId)
        expect(detail.body.data.current_registrations).toBe(1)

        const own = await runner.request('/api/registrations/user', {
          headers: authHeaders(token),
        })
        expect(own.response.status).toBe(200)
        expect(own.body.data).toEqual(expect.arrayContaining([
          expect.objectContaining({
            id: created.body.data.registrationId,
            eventId,
            eventName: 'Registration Contract Gala',
            eventDateTime: '2026-12-20T18:00:00.000Z',
            status: 'pending',
            paymentStatus: 'pending',
            specialRequests: 'Window seat',
          }),
        ]))

        const updated = await runner.request(
          `/api/registrations/${created.body.data.registrationId}`,
          {
            method: 'PUT',
            headers: authedJson(token),
            body: JSON.stringify({ specialRequests: 'Aisle seat' }),
          },
        )
        expect(updated.response.status).toBe(200)
        expect(updated.body).toEqual({ success: true, message: 'Registration updated successfully' })

        const registration = await runner.request(
          `/api/registrations/${created.body.data.registrationId}`,
          { headers: authHeaders(token) },
        )
        expect(registration.response.status).toBe(200)
        expect(registration.body.data).toMatchObject({ specialRequests: 'Aisle seat' })

        const duplicate = await register(token, eventId)
        expect(duplicate.response.status).toBe(400)
        expect(duplicate.body).toEqual({
          success: false,
          error: 'You are already registered for this event',
        })
      })

      it('waitlists at capacity without incrementing the event counter', async () => {
        const admin = await tokenFor(runner.seededCredentials)
        const member = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })
        const eventId = await createPublishedEvent(admin, 'Waitlist Contract Gala', 1)
        const counted = await register(admin, eventId)
        expect(counted.body.data.status).toBe('pending')

        const waitlisted = await register(member, eventId)
        expect(waitlisted.response.status).toBe(201)
        expect(waitlisted.body).toMatchObject({
          success: true,
          data: {
            registrationId: expect.any(Number),
            status: 'waitlisted',
            message: 'Event is full. You have been added to the waitlist.',
          },
        })
        expect((await eventDetail(admin, eventId)).body.data.current_registrations).toBe(1)

        const hidden = await runner.request(
          `/api/registrations/${counted.body.data.registrationId}`,
          { headers: authHeaders(member) },
        )
        expect(hidden.response.status).toBe(404)

        // Owner-or-admin: the owner and an administrator can both inspect it.
        const ownerView = await runner.request(
          `/api/registrations/${waitlisted.body.data.registrationId}`,
          { headers: authHeaders(member) },
        )
        expect(ownerView.response.status).toBe(200)
        const adminView = await runner.request(
          `/api/registrations/${waitlisted.body.data.registrationId}`,
          { headers: authHeaders(admin) },
        )
        expect(adminView.response.status).toBe(200)
      })

      it('cancels with a guarded decrement and atomically promotes the waitlist', async () => {
        const admin = await tokenFor(runner.seededCredentials)
        const member = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })

        const emptyEventId = await createPublishedEvent(
          admin,
          'Cancellation Counter Contract Gala',
          1,
          false,
        )
        const sole = await register(admin, emptyEventId)
        const cancelledSole = await runner.request(
          `/api/registrations/${sole.body.data.registrationId}`,
          { method: 'DELETE', headers: authHeaders(admin) },
        )
        expect(cancelledSole.response.status).toBe(200)
        expect(cancelledSole.body).toEqual({
          success: true,
          message: 'Registration cancelled successfully',
        })
        expect((await eventDetail(admin, emptyEventId)).body.data.current_registrations).toBe(0)

        const eventId = await createPublishedEvent(admin, 'Promotion Contract Gala', 1)
        const counted = await register(admin, eventId)
        const waiting = await register(member, eventId)
        expect(waiting.body.data.status).toBe('waitlisted')

        const cancelled = await runner.request(
          `/api/registrations/${counted.body.data.registrationId}`,
          { method: 'DELETE', headers: authHeaders(admin) },
        )
        expect(cancelled.response.status).toBe(200)
        expect((await eventDetail(admin, eventId)).body.data.current_registrations).toBe(1)

        const promoted = await runner.request(
          `/api/registrations/${waiting.body.data.registrationId}`,
          { headers: authHeaders(member) },
        )
        expect(promoted.response.status).toBe(200)
        expect(promoted.body.data.status).toBe('pending')
      })

      it('seeds participant access on register and lets an admin pay it forward', async () => {
        const admin = await tokenFor(runner.seededCredentials)
        const member = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })

        const eventId = await createPublishedEvent(admin, 'Payment Gate Contract Gala', 5)
        const registered = await register(member, eventId)

        // Register seeds the epa row as pending (cutover blocker #4).
        const seeded = await runner.request(
          `/api/events/${eventId}/participant-access`,
          { headers: authHeaders(member) },
        )
        expect(seeded.response.status).toBe(200)
        expect(seeded.body.data).toMatchObject({
          paymentStatus: 'pending',
          registrationStatus: 'pending',
        })

        // While pending, the paid-participants gate stays closed.
        const denied = await runner.request(
          `/api/events/${eventId}/participants`,
          { headers: authHeaders(member) },
        )
        expect(denied.response.status).toBe(403)

        // Only an admin can move the payment status forward.
        const forbidden = await runner.request(
          `/api/registrations/${registered.body.data.registrationId}/payment`,
          {
            method: 'POST',
            headers: authedJson(member),
            body: JSON.stringify({ paymentStatus: 'paid' }),
          },
        )
        expect(forbidden.response.status).toBe(403)

        const paid = await runner.request(
          `/api/registrations/${registered.body.data.registrationId}/payment`,
          {
            method: 'POST',
            headers: authedJson(admin),
            body: JSON.stringify({ paymentStatus: 'paid', paymentIntentId: 'pi_contract_2h' }),
          },
        )
        expect(paid.response.status).toBe(200)
        expect(paid.body).toEqual({
          success: true,
          message: 'Payment status updated successfully',
        })

        // The registration and the participant-access gate both flip.
        const own = await runner.request(
          `/api/registrations/${registered.body.data.registrationId}`,
          { headers: authHeaders(member) },
        )
        expect(own.body.data.paymentStatus).toBe('paid')

        const granted = await runner.request(
          `/api/events/${eventId}/participant-access`,
          { headers: authHeaders(member) },
        )
        expect(granted.response.status).toBe(200)
        expect(granted.body.data).toMatchObject({
          hasAccess: true,
          paymentStatus: 'paid',
        })

        const list = await runner.request(`/api/events/${eventId}/participants`, {
          headers: authHeaders(member),
        })
        expect(list.response.status).toBe(200)

        // Validation and unknown-id pins.
        const invalid = await runner.request(
          `/api/registrations/${registered.body.data.registrationId}/payment`,
          {
            method: 'POST',
            headers: authedJson(admin),
            body: JSON.stringify({ paymentStatus: 'teleported' }),
          },
        )
        expect(invalid.response.status).toBe(400)

        const missing = await runner.request('/api/registrations/424242/payment', {
          method: 'POST',
          headers: authedJson(admin),
          body: JSON.stringify({ paymentStatus: 'paid' }),
        })
        expect(missing.response.status).toBe(404)
        expect(missing.body).toEqual({ success: false, error: 'Registration not found' })
      })
    })
  }

  if (runner.participantsImplemented === true) {
    describe('participant privacy (Phase 2e)', () => {
      const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` })
      const authedJson = (token: string) => ({
        authorization: `Bearer ${token}`,
        'content-type': 'application/json',
      })
      const tokenFor = async (credentials: SeededCredentials) => {
        const login = await postJson('/api/auth/login', credentials)
        expect(login.response.status).toBe(200)
        return login.body.data.token as string
      }

      it('rejects an anonymous participant-list request', async () => {
        const result = await runner.request('/api/events/2/participants')
        expect(result.response.status).toBe(401)
        expect(result.body).toEqual({ success: false, error: 'Access token required' })
      })

      it('lets a paid registered participant view a privacy-masked list', async () => {
        const token = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })
        const access = await runner.request('/api/events/2/participant-access', {
          headers: authHeaders(token),
        })
        expect(access.response.status).toBe(200)
        expect(access.body).toMatchObject({
          success: true,
          data: {
            hasAccess: true,
            paymentRequired: false,
            paymentStatus: 'paid',
            registrationStatus: 'pending',
            accessLevel: {
              canViewParticipants: true,
              maxPrivacyLevelVisible: 3,
              canSeeContactInfo: false,
              canInitiateContact: true,
              accessLevel: 3,
            },
          },
        })

        const result = await runner.request('/api/events/2/participants?page=1&limit=20', {
          headers: authHeaders(token),
        })
        expect(result.response.status).toBe(200)
        expect(result.body).toMatchObject({
          success: true,
          data: {
            participants: [{
              id: 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
              displayName: 'Admin U.',
              membershipTier: 'Black Card',
              privacyLevel: 1,
              profession: 'Professional',
              interests: ['system administration'],
              ageRange: '40-44',
              canContact: true,
            }],
            totalCount: 1,
            paidParticipantCount: 2,
            unpaidParticipantCount: 0,
            participantCountByTier: { Platinum: 1, 'Black Card': 1 },
          },
          pagination: { page: 1, limit: 20, total: 1, totalPages: 1 },
        })
        expect(result.body.data.participants[0].company).toBeUndefined()
        expect(result.body.data.participants[0].bio).toBeUndefined()
        expect(result.body.data.participants[0].contactInfo).toBeUndefined()
      })

      it('sets and gets only the authenticated owner privacy override', async () => {
        const member = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })
        const admin = await tokenFor(runner.seededCredentials)
        const updated = await runner.request('/api/events/1/privacy-settings', {
          method: 'PUT',
          headers: authedJson(member),
          body: JSON.stringify({ privacyLevel: 2, allowContact: false, showInList: false }),
        })
        expect(updated.response.status).toBe(200)
        expect(updated.body).toEqual({
          success: true,
          message: 'Privacy settings updated successfully',
        })

        const own = await runner.request('/api/events/1/privacy-settings', {
          headers: authHeaders(member),
        })
        expect(own.response.status).toBe(200)
        expect(own.body).toEqual({
          success: true,
          data: { privacy_level: 2, allow_contact: false, show_in_list: false },
        })

        const otherUser = await runner.request('/api/events/1/privacy-settings', {
          headers: authHeaders(admin),
        })
        expect(otherUser.response.status).toBe(200)
        expect(otherUser.body).toEqual({
          success: true,
          data: { privacy_level: 5, allow_contact: true, show_in_list: true },
        })
      })

      it('denies participant access without a paid event access record', async () => {
        const token = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })
        const result = await runner.request('/api/events/1/participant-access', {
          headers: authHeaders(token),
        })
        expect(result.response.status).toBe(200)
        expect(result.body).toMatchObject({
          success: true,
          data: {
            hasAccess: false,
            paymentRequired: true,
            paymentStatus: 'none',
            registrationStatus: 'none',
            accessLevel: {
              canViewParticipants: false,
              maxPrivacyLevelVisible: 0,
              canInitiateContact: false,
            },
          },
        })

        const details = await runner.request(
          '/api/events/1/participants/f47ac10b-58cc-4372-a567-0e02b2c3d479',
          { headers: authHeaders(token) },
        )
        expect(details.response.status).toBe(403)
        expect(details.body).toEqual({
          success: false,
          error: 'Access denied - payment required to view participants',
        })
      })
    })
  }

  if (runner.salesImplemented === true) {
    // Reads, filters, updates, metrics, delete gating, and the read-only
    // pipeline/team lists run against the shared sales fixture
    // (backend-rust/d1/seed.sql plus its DuckDB mirror in
    // express.contract.test.ts) so both targets are pinned.
    describe('sales CRM (Phase 2f)', () => {
      const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` })
      const authedJson = (token: string) => ({
        authorization: `Bearer ${token}`,
        'content-type': 'application/json',
      })
      const tokenFor = async (credentials: SeededCredentials) => {
        const login = await postJson('/api/auth/login', credentials)
        expect(login.response.status).toBe(200)
        return login.body.data.token as string
      }
      const userIdOf = async (token: string) => {
        const validated = await runner.request('/api/auth/validate', {
          headers: authHeaders(token),
        })
        expect(validated.response.status).toBe(200)
        return validated.body.data.user.id as string
      }

      it('rejects every unauthenticated sales CRM route', async () => {
        const paths = [
          '/api/sales/leads',
          '/api/sales/leads/9001',
          '/api/sales/opportunities',
          '/api/sales/activities',
          '/api/sales/metrics',
          '/api/sales/pipeline/stages',
          '/api/sales/team',
        ]

        for (const path of paths) {
          const result = await runner.request(path)
          expect(result.response.status).toBe(401)
          expect(result.body).toEqual({ success: false, error: 'Access token required' })
        }

        const remove = await runner.request('/api/sales/leads/9003', { method: 'DELETE' })
        expect(remove.response.status).toBe(401)
        expect(remove.body).toEqual({ success: false, error: 'Access token required' })
      })

      it('lists leads with the assignee join, the filters, and the pagination envelope', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const adminId = await userIdOf(token)

        const list = await runner.request('/api/sales/leads?page=1&limit=50&status=new', {
          headers: authHeaders(token),
        })
        expect(list.response.status).toBe(200)
        expect(list.body).toMatchObject({
          success: true,
          pagination: {
            page: 1,
            limit: 50,
            total: expect.any(Number),
            totalPages: expect.any(Number),
          },
        })
        expect(list.body.data.map((row: any) => row.id)).toContain(9001)
        expect(
          list.body.data.every((row: any) => row.status === 'new'),
        ).toBe(true)
        expect(list.body.data[0]).toMatchObject({
          assigned_to_first_name: 'Admin',
          assigned_to_last_name: 'User',
        })

        const bySource = await runner.request('/api/sales/leads?source=event', {
          headers: authHeaders(token),
        })
        expect(bySource.response.status).toBe(200)
        expect(bySource.body.data.map((row: any) => row.id)).toEqual([9002])
        expect(bySource.body.pagination.total).toBe(1)
        expect(bySource.body.data[0]).toMatchObject({ status: 'closed_won', lead_score: 100 })

        const byAssignee = await runner.request(
          `/api/sales/leads?assignedTo=${encodeURIComponent(adminId)}&limit=50`,
          { headers: authHeaders(token) },
        )
        expect(byAssignee.response.status).toBe(200)
        expect(byAssignee.body.data.map((row: any) => row.id)).toContain(9001)
        expect(
          byAssignee.body.data.every((row: any) => row.assigned_to === adminId),
        ).toBe(true)

        const byTier = await runner.request('/api/sales/leads?membershipTier=Black%20Card', {
          headers: authHeaders(token),
        })
        expect(byTier.response.status).toBe(200)
        expect(
          byTier.body.data.every((row: any) => row.interested_membership_tier === 'Black Card'),
        ).toBe(true)

        const emptyPage = await runner.request('/api/sales/leads?status=closed_lost', {
          headers: authHeaders(token),
        })
        expect(emptyPage.body).toMatchObject({
          success: true,
          data: [],
          pagination: { total: 0, totalPages: 0 },
        })
      })

      it('rejects a negative limit with the driver-error envelope', async () => {
        // SQLite would treat a negative LIMIT as "no limit" (an unbounded
        // dump); DuckDB errors, so both targets answer the route's 500.
        const token = await tokenFor(runner.seededCredentials)
        const denied = await runner.request('/api/sales/leads?limit=-1', {
          headers: authHeaders(token),
        })
        expect(denied.response.status).toBe(500)
        expect(denied.body).toEqual({
          success: false,
          error: 'Failed to fetch sales leads',
        })
      })

      it('reads a single lead and 404s an unknown id', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const adminId = await userIdOf(token)

        const detail = await runner.request('/api/sales/leads/9001', {
          headers: authHeaders(token),
        })
        expect(detail.response.status).toBe(200)
        expect(detail.body).toEqual({
          success: true,
          data: expect.objectContaining({
            id: 9001,
            first_name: 'Seeded',
            last_name: 'Contract',
            email: 'crm-active@hesocial.test',
            company: 'Contract Holdings',
            job_title: 'Principal',
            annual_income: 25000000,
            net_worth: 120000000,
            source: 'referral',
            referral_code: 'CRM2F',
            lead_score: 100,
            status: 'new',
            interested_membership_tier: 'Black Card',
            budget_range: '5-10M',
            timeline: 'this-quarter',
            pain_points: 'Discreet networking',
            notes: 'Active contract lead',
            assigned_to: adminId,
            assigned_to_first_name: 'Admin',
            assigned_to_last_name: 'User',
          }),
        })

        const missing = await runner.request('/api/sales/leads/424242', {
          headers: authHeaders(token),
        })
        expect(missing.response.status).toBe(404)
        expect(missing.body).toEqual({ success: false, error: 'Lead not found' })
      })

      it('updates a lead through the writable-column whitelist', async () => {
        const token = await tokenFor(runner.seededCredentials)

        // Lead 9004 is deliberately childless: Express updates DuckDB rows by
        // delete-and-reinsert, so touching a lead that owns an opportunity
        // trips its own foreign key (pinned in the Rust-only block below).
        const updated = await runner.request('/api/sales/leads/9004', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({
            status: 'qualified',
            notes: 'Escalated after the pricing review',
            lead_score: 80,
          }),
        })
        expect(updated.response.status).toBe(200)
        expect(updated.body).toMatchObject({
          success: true,
          message: 'Lead updated successfully',
          data: {
            id: 9004,
            status: 'qualified',
            notes: 'Escalated after the pricing review',
            lead_score: 80,
          },
        })

        const detail = await runner.request('/api/sales/leads/9004', {
          headers: authHeaders(token),
        })
        expect(detail.body.data).toMatchObject({ status: 'qualified', lead_score: 80 })

        const missing = await runner.request('/api/sales/leads/424242', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ status: 'qualified' }),
        })
        expect(missing.response.status).toBe(404)
        expect(missing.body).toEqual({ success: false, error: 'Lead not found' })
      })

      it('moves an opportunity through the pipeline stages', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const adminId = await userIdOf(token)

        const list = await runner.request('/api/sales/opportunities?stage=proposal&limit=50', {
          headers: authHeaders(token),
        })
        expect(list.response.status).toBe(200)
        expect(list.body).toMatchObject({
          success: true,
          pagination: { page: 1, limit: 50, total: expect.any(Number) },
        })
        expect(list.body.data).toEqual(expect.arrayContaining([
          expect.objectContaining({
            id: 9102,
            lead_id: 9001,
            name: 'Diamond Membership Renewal',
            stage: 'proposal',
            probability: 60,
            value: 480000,
            membership_tier: 'Diamond',
            payment_terms: 'semi-annual',
            assigned_to: adminId,
            lead_first_name: 'Seeded',
            lead_last_name: 'Contract',
            lead_email: 'crm-active@hesocial.test',
            assigned_to_first_name: 'Admin',
          }),
        ]))

        // Opportunity 9103 has no activity child: Express rewrites the row by
        // delete-and-reinsert, so a referenced opportunity trips its own key.
        const before = await runner.request('/api/sales/opportunities?stage=negotiation', {
          headers: authHeaders(token),
        })
        expect(
          before.body.data.map((row: any) => row.id),
        ).toEqual([9103])

        const transition = await runner.request('/api/sales/opportunities/9103', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ stage: 'closed_won', probability: 100 }),
        })
        expect(transition.response.status).toBe(200)
        expect(transition.body).toMatchObject({
          success: true,
          message: 'Opportunity updated successfully',
          data: { id: 9103, stage: 'closed_won', probability: 100, value: 120000 },
        })

        const retired = await runner.request('/api/sales/opportunities?stage=negotiation', {
          headers: authHeaders(token),
        })
        expect(retired.body.data).toEqual([])

        const missing = await runner.request('/api/sales/opportunities/424242', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ stage: 'closed_won' }),
        })
        expect(missing.response.status).toBe(404)
        expect(missing.body).toEqual({ success: false, error: 'Opportunity not found' })
      })

      it('lists activities and answers without a pagination envelope', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const adminId = await userIdOf(token)

        const byLead = await runner.request('/api/sales/activities?leadId=9002&limit=10', {
          headers: authHeaders(token),
        })
        expect(byLead.response.status).toBe(200)
        expect(byLead.body.pagination).toBeUndefined()
        expect(byLead.body.data).toEqual(expect.arrayContaining([
          expect.objectContaining({
            id: 9201,
            lead_id: 9002,
            opportunity_id: 9101,
            activity_type: 'meeting',
            subject: 'Founding seat presentation',
            outcome: 'signed',
            duration_minutes: 60,
            created_by: adminId,
            created_by_first_name: 'Admin',
            created_by_last_name: 'User',
          }),
        ]))

        const byOpportunity = await runner.request('/api/sales/activities?opportunityId=9102', {
          headers: authHeaders(token),
        })
        expect(
          byOpportunity.body.data.map((row: any) => row.id),
        ).toEqual([9202])

        const paged = await runner.request('/api/sales/activities?page=2&limit=2', {
          headers: authHeaders(token),
        })
        expect(paged.body.data).toEqual([])
      })

      it('aggregates the dashboard metrics per reporting period', async () => {
        const token = await tokenFor(runner.seededCredentials)

        const monthly = await runner.request('/api/sales/metrics', { headers: authHeaders(token) })
        expect(monthly.response.status).toBe(200)
        expect(monthly.body).toMatchObject({
          success: true,
          data: {
            totalLeads: expect.any(Number),
            qualifiedLeads: expect.any(Number),
            totalOpportunities: expect.any(Number),
            totalPipelineValue: expect.any(Number),
            conversionRate: expect.any(Number),
            averageDealSize: expect.any(Number),
            // Pinned live behavior: the cycle length is a hard-coded default
            // and all three revenue buckets echo the same won revenue.
            salesCycleLength: 30,
            winRate: expect.any(Number),
            monthlyRevenue: expect.any(Number),
            quarterlyRevenue: expect.any(Number),
            yearlyRevenue: expect.any(Number),
          },
        })
        expect(monthly.body.data.totalLeads).toBeGreaterThanOrEqual(1)
        expect(monthly.body.data.qualifiedLeads).toBeGreaterThanOrEqual(1)

        const yearly = await runner.request('/api/sales/metrics?period=yearly', {
          headers: authHeaders(token),
        })
        expect(yearly.response.status).toBe(200)
        expect(yearly.body.data.totalLeads).toBeGreaterThanOrEqual(monthly.body.data.totalLeads)
        expect(yearly.body.data.totalOpportunities).toBeGreaterThanOrEqual(1)

        // An unsupported period string drops the date filter entirely, so the
        // 2020 fixture rows only surface in this bucket.
        const unfiltered = await runner.request('/api/sales/metrics?period=weekly', {
          headers: authHeaders(token),
        })
        expect(unfiltered.response.status).toBe(200)
        expect(unfiltered.body.data.totalLeads).toBeGreaterThanOrEqual(
          yearly.body.data.totalLeads + 1,
        )
        expect(unfiltered.body.data.totalOpportunities).toBeGreaterThanOrEqual(2)
        expect(unfiltered.body.data.totalPipelineValue).toBeGreaterThanOrEqual(730000)
        expect(unfiltered.body.data.wonOpportunities).toBeUndefined()
        expect(unfiltered.body.data.conversionRate).toBeGreaterThanOrEqual(0)
        expect(unfiltered.body.data.winRate).toBeLessThanOrEqual(100)
      })

      it('serves only the active pipeline stages in display order', async () => {
        const token = await tokenFor(runner.seededCredentials)

        const stages = await runner.request('/api/sales/pipeline/stages', {
          headers: authHeaders(token),
        })
        expect(stages.response.status).toBe(200)
        expect(stages.body).toEqual({ success: true, data: expect.any(Array) })
        expect(stages.body.data.map((row: any) => row.name)).toEqual([
          'qualification',
          'needs_analysis',
          'proposal',
          'negotiation',
        ])
        expect(stages.body.data[0]).toMatchObject({
          id: 9401,
          display_order: 1,
          default_probability: 25,
          is_active: true,
        })
      })

      it('serves only the active sales team members with their joined profiles', async () => {
        const token = await tokenFor(runner.seededCredentials)

        const team = await runner.request('/api/sales/team', { headers: authHeaders(token) })
        expect(team.response.status).toBe(200)
        expect(team.body.success).toBe(true)
        expect(team.body.data).toHaveLength(1)
        expect(team.body.data[0]).toMatchObject({
          id: 9301,
          role: 'sales_rep',
          territory: 'Taipei',
          quota_amount: 3000000,
          is_active: true,
          first_name: 'Admin',
          last_name: 'User',
          email: 'admin@hesocial.com',
          manager_first_name: 'Test',
          manager_last_name: 'Platinum',
        })
      })

      it('admin-gates lead deletion and pins the rowCount blind spot', async () => {
        const admin = await tokenFor(runner.seededCredentials)
        const member = await tokenFor({
          email: 'test.platinum@example.com',
          password: 'test123',
        })

        const forbidden = await runner.request('/api/sales/leads/9003', {
          method: 'DELETE',
          headers: authHeaders(member),
        })
        expect(forbidden.response.status).toBe(403)
        expect(forbidden.body).toEqual({ success: false, error: 'Admin access required' })

        // Known live drift: the Express delete reads `result.rowCount`, which the
        // DuckDB adapter never populates, so a no-match delete still reports 200.
        const ghost = await runner.request('/api/sales/leads/424242', {
          method: 'DELETE',
          headers: authHeaders(admin),
        })
        expect(ghost.response.status).toBe(200)
        expect(ghost.body).toEqual({ success: true, message: 'Lead deleted successfully' })

        const deleted = await runner.request('/api/sales/leads/9003', {
          method: 'DELETE',
          headers: authHeaders(admin),
        })
        expect(deleted.response.status).toBe(200)
        expect(deleted.body).toEqual({ success: true, message: 'Lead deleted successfully' })

        const gone = await runner.request('/api/sales/leads/9003', {
          headers: authHeaders(admin),
        })
        expect(gone.response.status).toBe(404)
        expect(gone.body).toEqual({ success: false, error: 'Lead not found' })
      })
    })
  }

  if (runner.salesFlowImplemented === true) {
    // Sales paths the Express target cannot execute at all today, so they are
    // pinned against the Rust target only:
    //   * INSERTs 500 because DuckDB gives `id INTEGER PRIMARY KEY` no implicit
    //     sequence, so createLead/createOpportunity/createActivity always fail
    //     with `NOT NULL constraint failed: sales_leads.id`;
    //   * the leads `search` filter and the opportunities `membershipTier`
    //     filter 500 on unqualified column names that are ambiguous against the
    //     joined `users` row;
    //   * updating a lead that owns an opportunity 500s because DuckDB applies
    //     the child foreign key to the delete-and-reinsert of the parent row;
    //   * D1 declares CHECK vocabularies and NOT NULL columns DuckDB lacks.
    describe('sales CRM intended flow (Phase 2f)', () => {
      const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` })
      const authedJson = (token: string) => ({
        authorization: `Bearer ${token}`,
        'content-type': 'application/json',
      })
      let sequence = 0
      const tokenFor = async (credentials: SeededCredentials) => {
        const login = await postJson('/api/auth/login', credentials)
        expect(login.response.status).toBe(200)
        return login.body.data.token as string
      }
      const userIdOf = async (token: string) => {
        const validated = await runner.request('/api/auth/validate', {
          headers: authHeaders(token),
        })
        return validated.body.data.user.id as string
      }
      const leadPayload = (userId: string, overrides: Record<string, unknown> = {}) => {
        sequence += 1
        return {
          firstName: 'Crm',
          lastName: 'Contract',
          email: `sales-write-${Date.now()}-${sequence}@hesocial.test`,
          phone: '+886900000123',
          company: 'Contract Holdings',
          jobTitle: 'Principal',
          annualIncome: 25000000,
          netWorth: 120000000,
          source: 'referral',
          referralCode: 'CRM2F',
          interestedMembershipTier: 'Black Card',
          budgetRange: '5-10M',
          timeline: 'this-quarter',
          painPoints: 'Discreet networking',
          interests: ['fine dining', 'yachting'],
          notes: 'Characterization lead',
          assignedTo: userId,
          ...overrides,
        }
      }
      const createLead = async (token: string, userId: string) => {
        const created = await runner.request('/api/sales/leads', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify(leadPayload(userId)),
        })
        expect(created.response.status).toBe(201)
        return created.body.data.id as number
      }

      it('creates a lead and applies the income/net-worth/tier score', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const userId = await userIdOf(token)

        const created = await runner.request('/api/sales/leads', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify(leadPayload(userId)),
        })
        expect(created.response.status).toBe(201)
        expect(created.body.message).toBe('Lead created successfully')
        expect(created.body.data).toMatchObject({
          first_name: 'Crm',
          last_name: 'Contract',
          source: 'referral',
          status: 'new',
          // 40 (income >= 20M) + 40 (net worth >= 100M) + 20 (Black Card tier).
          lead_score: 100,
          interests: '["fine dining","yachting"]',
          assigned_to: userId,
        })
        expect(created.body.data.id).toEqual(expect.any(Number))

        const detail = await runner.request(`/api/sales/leads/${created.body.data.id}`, {
          headers: authHeaders(token),
        })
        expect(detail.response.status).toBe(200)
        expect(detail.body.data.assigned_to_first_name).toBe('Admin')

        const updated = await runner.request(`/api/sales/leads/${created.body.data.id}`, {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ status: 'nurturing', next_follow_up_date: '2026-09-14' }),
        })
        expect(updated.response.status).toBe(200)
        expect(updated.body).toMatchObject({
          success: true,
          message: 'Lead updated successfully',
          data: { status: 'nurturing' },
        })

        // The worker also resolves the camelCase spellings the frontend types
        // use; Express would have interpolated `leadScore` as a column name.
        const camel = await runner.request(`/api/sales/leads/${created.body.data.id}`, {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ leadScore: 60, budgetRange: '3-5M' }),
        })
        expect(camel.response.status).toBe(200)
        expect(camel.body.data).toMatchObject({
          lead_score: 60,
          budget_range: '3-5M',
          status: 'nurturing',
        })
      })

      it('scores a mid-tier lead on the tier signal alone', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const userId = await userIdOf(token)

        const created = await runner.request('/api/sales/leads', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify(leadPayload(userId, {
            annualIncome: 6000000,
            netWorth: 12000000,
            interestedMembershipTier: 'Diamond',
            interests: [],
          })),
        })
        expect(created.response.status).toBe(201)
        expect(created.body.data).toMatchObject({
          lead_score: 15,
          interests: '[]',
        })
      })

      it('filters leads by the free-text search the Express route cannot run', async () => {
        const token = await tokenFor(runner.seededCredentials)

        const bySearch = await runner.request('/api/sales/leads?search=seeded', {
          headers: authHeaders(token),
        })
        expect(bySearch.response.status).toBe(200)
        expect(bySearch.body.data.map((row: any) => row.id)).toContain(9001)
      })

      it('creates an opportunity for a lead', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const userId = await userIdOf(token)
        const leadId = await createLead(token, userId)

        const created = await runner.request('/api/sales/opportunities', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify({
            leadId,
            name: 'Contract Platinum Membership',
            description: 'Created through the contract',
            stage: 'qualification',
            probability: 30,
            value: 1250000,
            expectedCloseDate: '2026-12-01',
            membershipTier: 'Platinum',
            paymentTerms: 'annual-prepaid',
            assignedTo: userId,
          }),
        })
        expect(created.response.status).toBe(201)
        expect(created.body).toMatchObject({
          success: true,
          message: 'Opportunity created successfully',
          data: {
            lead_id: leadId,
            name: 'Contract Platinum Membership',
            stage: 'qualification',
            probability: 30,
            value: 1250000,
            membership_tier: 'Platinum',
            assigned_to: userId,
          },
        })

        const list = await runner.request('/api/sales/opportunities?stage=qualification&limit=50', {
          headers: authHeaders(token),
        })
        expect(
          list.body.data.some((row: any) => row.id === created.body.data.id),
        ).toBe(true)

        const byTier = await runner.request('/api/sales/opportunities?membershipTier=Black%20Card', {
          headers: authHeaders(token),
        })
        expect(byTier.response.status).toBe(200)
        expect(byTier.body.data.map((row: any) => row.id)).toEqual([9101])

        const byAssignee = await runner.request(
          `/api/sales/opportunities?assignedTo=${encodeURIComponent(userId)}&limit=50`,
          { headers: authHeaders(token) },
        )
        expect(byAssignee.response.status).toBe(200)
        expect(byAssignee.body.data.map((row: any) => row.id)).toContain(9101)
        expect(
          byAssignee.body.data.every((row: any) => row.assigned_to === userId),
        ).toBe(true)
      })

      it('survives its lead being deleted (orphaned with NULL lead_id, no FK action)', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const userId = await userIdOf(token)
        const leadId = await createLead(token, userId)

        const created = await runner.request('/api/sales/opportunities', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify({
            leadId,
            name: 'Orphan Check Membership',
            description: 'Created through the contract',
            stage: 'qualification',
            probability: 30,
            value: 900000,
            expectedCloseDate: '2026-12-01',
            membershipTier: 'Platinum',
            paymentTerms: 'annual-prepaid',
            assignedTo: userId,
          }),
        })
        expect(created.response.status).toBe(201)
        const opportunityId = created.body.data.id

        // D1 enforces its foreign keys, so the delete handler orphans the
        // opportunity explicitly (batch, lead_id -> NULL): the row must
        // survive the delete — not be destroyed (the pre-fix CASCADE) — and
        // the opportunities list must render it (OpportunityRow.lead_id had
        // to become Option: a NULL row hung the whole request).
        const deleted = await runner.request(`/api/sales/leads/${leadId}`, {
          method: 'DELETE',
          headers: authHeaders(token),
        })
        expect(deleted.response.status).toBe(200)

        const goneLead = await runner.request(`/api/sales/leads/${leadId}`, {
          headers: authHeaders(token),
        })
        expect(goneLead.response.status).toBe(404)

        const list = await runner.request('/api/sales/opportunities', {
          headers: authHeaders(token),
        })
        expect(list.response.status).toBe(200)
        const orphan = list.body.data.find((row: any) => row.id === opportunityId)
        expect(orphan).toBeDefined()
        expect(orphan.lead_id).toBeNull()
      })

      it('records an activity against the authenticated caller', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const userId = await userIdOf(token)
        const leadId = await createLead(token, userId)

        const created = await runner.request('/api/sales/activities', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify({
            leadId,
            opportunityId: null,
            activityType: 'call',
            subject: 'Intro call',
            description: 'Walked through the Diamond tier',
            outcome: 'reached',
            durationMinutes: 15,
            scheduledAt: '2026-08-31T02:00:00.000Z',
            completedAt: '2026-08-31T02:15:00.000Z',
          }),
        })
        expect(created.response.status).toBe(201)
        expect(created.body).toMatchObject({
          success: true,
          message: 'Activity created successfully',
          data: {
            lead_id: leadId,
            activity_type: 'call',
            subject: 'Intro call',
            outcome: 'reached',
            duration_minutes: 15,
            created_by: userId,
          },
        })

        const list = await runner.request(`/api/sales/activities?leadId=${leadId}`, {
          headers: authHeaders(token),
        })
        expect(list.body.data.map((row: any) => row.id)).toEqual([created.body.data.id])
      })

      it('surfaces the D1 constraint vocabulary through the Express error envelope', async () => {
        const token = await tokenFor(runner.seededCredentials)
        const userId = await userIdOf(token)

        // D1 declares sales_leads.source NOT NULL where DuckDB leaves it
        // nullable, and adds CHECK vocabularies DuckDB never had. Both land on
        // the same 500 envelope Express uses for its own bind errors.
        const missingSource = await runner.request('/api/sales/leads', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify(leadPayload(userId, { source: undefined })),
        })
        expect(missingSource.response.status).toBe(500)
        expect(missingSource.body).toEqual({ success: false, error: 'Failed to create lead' })

        const badTier = await runner.request('/api/sales/leads', {
          method: 'POST',
          headers: authedJson(token),
          body: JSON.stringify(leadPayload(userId, { interestedMembershipTier: 'Gold' })),
        })
        expect(badTier.response.status).toBe(500)
        expect(badTier.body).toEqual({ success: false, error: 'Failed to create lead' })

        const badStatus = await runner.request('/api/sales/leads/9004', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ status: 'hot' }),
        })
        expect(badStatus.response.status).toBe(500)
        expect(badStatus.body).toEqual({ success: false, error: 'Failed to update lead' })

        const badScore = await runner.request('/api/sales/leads/9004', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ lead_score: 900 }),
        })
        expect(badScore.response.status).toBe(500)
        expect(badScore.body).toEqual({ success: false, error: 'Failed to update lead' })

        // `salesRepId` reaches the query as a bound value; Express interpolates
        // it into the SQL string, which cannot carry a UUID at all.
        const mine = await runner.request(
          `/api/sales/metrics?period=weekly&salesRepId=${encodeURIComponent(userId)}`,
          { headers: authedJson(token) },
        )
        expect(mine.response.status).toBe(200)
        expect(mine.body.data.totalLeads).toBeGreaterThanOrEqual(3)

        const nobody = await runner.request('/api/sales/metrics?period=weekly&salesRepId=999999', {
          headers: authedJson(token),
        })
        expect(nobody.response.status).toBe(200)
        expect(nobody.body.data.totalLeads).toBe(0)
        expect(nobody.body.data.totalOpportunities).toBe(0)
        expect(nobody.body.data.conversionRate).toBe(0)
        expect(nobody.body.data.winRate).toBe(0)

        // Lead 9002 owns an opportunity. Express 500s on this update with a
        // foreign-key error; D1 updates the parent row in place.
        const parentUpdate = await runner.request('/api/sales/leads/9002', {
          method: 'PUT',
          headers: authedJson(token),
          body: JSON.stringify({ notes: 'Reparented without tripping the child key' }),
        })
        expect(parentUpdate.response.status).toBe(200)
        expect(parentUpdate.body.data).toMatchObject({
          id: 9002,
          notes: 'Reparented without tripping the child key',
        })
      })
    })
  }
}
