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
  adminStatsExpectation?: 'authenticated' | 'not-implemented'
}

export const defineContractTests = (runner: ContractRunner): void => {
  const postJson = (path: string, body: unknown) => runner.request(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  const authTest = it.skipIf(runner.authImplemented === false)

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
}
