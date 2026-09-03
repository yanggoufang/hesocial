import { describe, expect, it } from 'vitest'
import type { ContractRequest, SeededCredentials } from './api.contract.js'

export interface AnalyticsContractRunner {
  request: ContractRequest
  seededCredentials: SeededCredentials
  analyticsImplemented?: boolean
  trackingRequiresAdmin: boolean
}

/**
 * Phase 2g's final 12-route analytics contract.
 *
 * Analytics Engine reads (6):
 *   GET /api/analytics/visitors
 *   GET /api/analytics/visitors/daily
 *   GET /api/analytics/pages/popular
 *   GET /api/analytics/conversion
 *   GET /api/analytics/visitors/:visitorId
 *   GET /api/analytics/events/engagement
 * D1 reads (5):
 *   GET /api/analytics/events/overview
 *   GET /api/analytics/events/performance
 *   GET /api/analytics/events/:id/performance
 *   GET /api/analytics/revenue/events
 *   GET /api/analytics/engagement/members
 * Tracking beacon (1):
 *   POST /api/analytics/events/track
 *
 * Express still admin-guards the beacon with the rest of its router. The Rust
 * port deliberately exposes that one write route because the frontend beacon
 * calls it without a token; every read remains admin-only on both targets.
 */
export const defineAnalyticsContractTests = (runner: AnalyticsContractRunner): void => {
  const analyticsTest = it.skipIf(runner.analyticsImplemented !== true)
  const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` })
  const jsonHeaders = (token?: string) => ({
    ...(token ? authHeaders(token) : {}),
    'content-type': 'application/json',
  })
  const tokenFor = async (credentials: SeededCredentials) => {
    const login = await runner.request('/api/auth/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(credentials),
    })
    expect(login.response.status).toBe(200)
    return login.body.data.token as string
  }

  const aeReads = [
    '/api/analytics/visitors?days=365',
    '/api/analytics/visitors/daily?days=365',
    '/api/analytics/pages/popular?limit=20',
    '/api/analytics/conversion?days=365',
    '/api/analytics/visitors/visitor_contract',
    '/api/analytics/events/engagement?days=365',
  ]
  const d1Reads = [
    '/api/analytics/events/overview?days=365',
    '/api/analytics/events/performance?days=365',
    '/api/analytics/events/2/performance',
    '/api/analytics/revenue/events',
    '/api/analytics/engagement/members',
  ]
  const allReads = [...aeReads, ...d1Reads]

  describe('analytics engine contract (Phase 2g)', () => {
    analyticsTest('admin-guards all reads and rejects a signed-in non-admin', async () => {
      const memberToken = await tokenFor({
        email: 'test.platinum@example.com',
        password: 'test123',
      })

      for (const path of allReads) {
        const anonymous = await runner.request(path)
        expect(anonymous.response.status).toBe(401)
        expect(anonymous.body).toMatchObject({ success: false })

        const member = await runner.request(path, { headers: authHeaders(memberToken) })
        expect(member.response.status).toBe(403)
        expect(member.body).toMatchObject({
          success: false,
          error: 'Admin access required',
        })
      }
    })

    analyticsTest('returns the six Analytics-Engine read envelopes', async () => {
      const token = await tokenFor(runner.seededCredentials)
      const responses = await Promise.all(
        aeReads.map(path => runner.request(path, { headers: authHeaders(token) })),
      )

      for (const result of responses) {
        expect(result.response.status).toBe(200)
        expect(result.body).toMatchObject({ success: true })
      }

      expect(responses[0].body.data).toMatchObject({
        period_days: 365,
        unique_visitors: expect.any(Number),
        total_page_views: expect.any(Number),
        converted_visitors: expect.any(Number),
        avg_pages_per_visitor: expect.any(Number),
        new_visitors: expect.any(Number),
      })
      expect(responses[1].body.data).toEqual(expect.any(Array))
      expect(responses[1].body.data[0]).toMatchObject({
        date: expect.anything(),
        unique_visitors: expect.any(Number),
        total_page_views: expect.any(Number),
        converted_visitors: expect.any(Number),
        avg_pages_per_visitor: expect.any(Number),
      })
      expect(responses[2].body.data).toEqual(expect.any(Array))
      expect(responses[2].body.data[0]).toMatchObject({
        views: expect.any(Number),
        unique_visitors: expect.any(Number),
        conversion_rate: expect.any(Number),
      })
      expect(responses[2].body.data[0]).toHaveProperty('path')
      expect(responses[3].body.data).toMatchObject({
        period_days: 365,
        total_visitors: expect.any(Number),
        event_viewers: expect.any(Number),
        registered_users: expect.any(Number),
        conversion_rate: expect.any(Number),
      })
      expect(responses[4].body.data).toMatchObject({
        session: expect.any(Object),
        page_views: expect.any(Array),
      })
      expect(responses[5].body.data).toMatchObject({
        period_days: 365,
        engagement: expect.any(Array),
      })
      expect(responses[5].body.data.engagement[0]).toMatchObject({
        date: expect.anything(),
        unique_visitors: expect.any(Number),
        total_page_views: expect.any(Number),
        event_page_views: expect.any(Number),
        registration_page_views: expect.any(Number),
        avg_time_spent: expect.any(Number),
      })
    })

    analyticsTest('returns the five D1-backed envelopes from the shared event fixture', async () => {
      const token = await tokenFor(runner.seededCredentials)
      const responses = await Promise.all(
        d1Reads.map(path => runner.request(path, { headers: authHeaders(token) })),
      )

      for (const result of responses) {
        expect(result.response.status).toBe(200)
        expect(result.body).toMatchObject({ success: true })
      }

      expect(responses[0].body.data).toMatchObject({
        period_days: 365,
        event_stats: {
          total_events: expect.any(Number),
          recent_events: expect.any(Number),
          upcoming_events: expect.any(Number),
          past_events: expect.any(Number),
        },
        registration_stats: {
          total_registrations: expect.any(Number),
          recent_registrations: expect.any(Number),
          unique_attendees: expect.any(Number),
        },
        popular_events: expect.any(Array),
      })
      expect(responses[1].body.data).toMatchObject({
        period_days: 365,
        events: expect.any(Array),
      })
      expect(responses[1].body.data.events).toEqual(expect.arrayContaining([
        expect.objectContaining({
          id: 2,
          name: 'Autumn Yacht Social',
          capacity: 30,
          current_attendees: 1,
          occupancy_rate: expect.any(Number),
          total_registrations: expect.any(Number),
        }),
      ]))
      expect(responses[2].body.data).toMatchObject({
        event: {
          id: 2,
          category_name: '遊艇派對',
          venue_name: 'Keelung Luxury Yacht',
          fill_rate: expect.any(Number),
          current_revenue: expect.any(Number),
          potential_revenue: expect.any(Number),
        },
        registrationTimeline: expect.any(Array),
        membershipBreakdown: expect.any(Array),
        statusBreakdown: expect.any(Array),
      })
      expect(responses[3].body.data).toMatchObject({
        monthlyRevenue: expect.any(Array),
        categoryRevenue: expect.any(Array),
        tierRevenue: expect.any(Array),
      })
      expect(responses[3].body.data.monthlyRevenue[0]).toMatchObject({
        month: '2025-07',
        event_count: expect.any(Number),
        total_registrations: expect.any(Number),
        revenue: expect.any(Number),
      })
      expect(responses[4].body.data).toMatchObject({
        engagement: expect.any(Array),
        topMembers: expect.any(Array),
        retention: expect.any(Array),
      })
      expect(responses[4].body.data.retention[0]).toMatchObject({
        cohort_month: '2025-07',
        cohort_size: expect.any(Number),
        active_this_month: expect.any(Number),
        retention_rate: expect.any(Number),
      })
    })

    analyticsTest('smokes the tracking beacon and pins its target-specific auth contract', async () => {
      const adminToken = await tokenFor(runner.seededCredentials)
      const memberToken = await tokenFor({
        email: 'test.platinum@example.com',
        password: 'test123',
      })
      const body = JSON.stringify({
        visitor_id: 'visitor_contract',
        event_type: 'contract_smoke',
        event_data: { route: '/events/2' },
      })

      if (runner.trackingRequiresAdmin) {
        const anonymous = await runner.request('/api/analytics/events/track', {
          method: 'POST',
          headers: jsonHeaders(),
          body,
        })
        expect(anonymous.response.status).toBe(401)
        expect(anonymous.body).toMatchObject({ success: false })

        const member = await runner.request('/api/analytics/events/track', {
          method: 'POST',
          headers: jsonHeaders(memberToken),
          body,
        })
        expect(member.response.status).toBe(403)
        expect(member.body).toMatchObject({
          success: false,
          error: 'Admin access required',
        })
      }

      const tracked = await runner.request('/api/analytics/events/track', {
        method: 'POST',
        headers: jsonHeaders(runner.trackingRequiresAdmin ? adminToken : undefined),
        body,
      })
      expect(tracked.response.status).toBe(200)
      expect(tracked.body).toEqual({
        success: true,
        message: 'Event tracked successfully',
      })

      const invalid = await runner.request('/api/analytics/events/track', {
        method: 'POST',
        headers: jsonHeaders(runner.trackingRequiresAdmin ? adminToken : undefined),
        body: JSON.stringify({ visitor_id: 'visitor_contract' }),
      })
      expect(invalid.response.status).toBe(400)
      expect(invalid.body).toEqual({
        success: false,
        error: 'visitor_id and event_type are required',
      })
    })
  })
}
