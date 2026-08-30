import { IncomingMessage, ServerResponse, type Server } from 'node:http'
import { mkdtemp, readFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { Duplex } from 'node:stream'
import { fileURLToPath } from 'node:url'
import type { Express } from 'express'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'

let server: Server | undefined
let baseUrl: string | undefined
let expressApp: Express
let tempDirectory: string
let databaseModule: typeof import('../src/database/duckdb-connection.js')

const requestInProcess = async (path: string, init?: RequestInit) => {
  const requestBody = typeof init?.body === 'string' ? Buffer.from(init.body) : Buffer.alloc(0)
  const headers: Record<string, string> = {}
  new Headers(init?.headers).forEach((value, name) => {
    headers[name.toLowerCase()] = value
  })
  if (requestBody.length > 0) {
    headers['content-length'] = String(requestBody.length)
  }

  const output: Buffer[] = []
  const socket = new Duplex({
    read() {},
    write(chunk, _encoding, callback) {
      output.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk))
      callback()
    },
  })
  Object.defineProperties(socket, {
    remoteAddress: { value: '127.0.0.1' },
    remotePort: { value: 12345 },
  })
  const incoming = new IncomingMessage(socket)
  incoming.method = init?.method || 'GET'
  incoming.url = path
  incoming.headers = headers
  incoming.httpVersion = '1.1'
  incoming.httpVersionMajor = 1
  incoming.httpVersionMinor = 1
  incoming.push(requestBody.length > 0 ? requestBody : null)
  if (requestBody.length > 0) {
    incoming.push(null)
  }

  const outgoing = new ServerResponse(incoming)
  outgoing.assignSocket(socket)
  let directBody: Buffer | undefined
  await new Promise<void>((resolve, reject) => {
    outgoing.once('error', reject)
    const originalEnd = outgoing.end.bind(outgoing)
    outgoing.end = ((...args: unknown[]) => {
      const chunk = args[0]
      if (typeof chunk === 'string' || Buffer.isBuffer(chunk)) {
        directBody = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)
      }
      const result = originalEnd(...args)
      setImmediate(resolve)
      return result
    }) as typeof outgoing.end
    expressApp(incoming, outgoing)
  })

  const rawResponse = Buffer.concat(output)
  const bodyOffset = rawResponse.indexOf('\r\n\r\n') + 4
  const serializedBody = directBody?.toString('utf8') || rawResponse.subarray(bodyOffset).toString('utf8')
  return {
    body: serializedBody ? JSON.parse(serializedBody) : undefined,
    response: { status: outgoing.statusCode },
  }
}

const request = async (path: string, init?: RequestInit) => {
  if (baseUrl) {
    const response = await fetch(`${baseUrl}${path}`, init)
    const body = await response.json()
    return { body, response: { status: response.status } }
  }

  return requestInProcess(path, init)
}

const postJson = (path: string, body: unknown) => request(path, {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify(body),
})

const makeSchemaDuckDbCompatible = (schema: string): string => schema
  // DuckDB does not support referential actions in FOREIGN KEY declarations.
  .replace(/ ON DELETE (?:CASCADE|SET NULL|SET DEFAULT)/g, '')
  // Registration creates UUID user IDs, while the historical schema used INTEGER.
  .replace(/(CREATE TABLE IF NOT EXISTS users \(\n\s*)id INTEGER/, '$1id VARCHAR')
  .replace(
    /\b(user_id|organizer_id|verified_by|assigned_to|created_by|viewer_id|participant_id|manager_id) INTEGER\b/g,
    '$1 VARCHAR',
  )

beforeAll(async () => {
  tempDirectory = await mkdtemp(join(tmpdir(), 'hesocial-vitest-'))
  process.env.DUCKDB_PATH = join(tempDirectory, 'hesocial-test.duckdb')

  databaseModule = await import('../src/database/duckdb-connection.js')

  const schemaPath = fileURLToPath(new URL('../../database/duckdb-schema.sql', import.meta.url))
  const schema = makeSchemaDuckDbCompatible(await readFile(schemaPath, 'utf8'))

  await databaseModule.duckdb.connect()
  await databaseModule.duckdb.query(schema)
  await databaseModule.connectDatabases()
  await databaseModule.ensureSeedUsers()

  // ensureSeedUsers currently provides superadmin@hesocial.com; retain the
  // requested legacy login alias only in this isolated characterization DB.
  await databaseModule.duckdb.query(`
    INSERT OR IGNORE INTO users (
      id, email, password_hash, first_name, last_name, age, profession,
      annual_income, net_worth, membership_tier, privacy_level,
      is_verified, verification_status, role, bio, interests,
      created_at, updated_at
    ) VALUES (
      '1000',
      'admin@hesocial.com',
      '$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6',
      'Admin',
      'User',
      40,
      'System Administrator',
      5000000,
      30000000,
      'Black Card',
      5,
      true,
      'approved',
      'super_admin',
      'Legacy administrator test account.',
      ['system administration'],
      CURRENT_TIMESTAMP,
      CURRENT_TIMESTAMP
    )
  `)
  await databaseModule.duckdb.query(`
    INSERT OR IGNORE INTO server_state (id, start_count, first_start_time, last_start_time)
    VALUES (1, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
  `)

  const [{ default: app }, { default: createRoutes }] = await Promise.all([
    import('../src/server.js'),
    import('../src/routes/main.js'),
  ])
  expressApp = app
  app.use('/api', await createRoutes())

  try {
    server = await new Promise<Server>((resolve, reject) => {
      const listeningServer = app.listen(0, '127.0.0.1', () => resolve(listeningServer))
      listeningServer.once('error', reject)
    })

    const address = server.address()
    if (!address || typeof address === 'string') {
      throw new Error('Expected the test server to listen on an ephemeral TCP port')
    }
    baseUrl = `http://127.0.0.1:${address.port}`
  } catch (error) {
    if (!(error instanceof Error) || !('code' in error) || error.code !== 'EPERM') {
      throw error
    }
    // Some restricted CI sandboxes prohibit all TCP listeners. The request
    // helper drives the same exported Express app through Node HTTP objects.
  }
})

afterAll(async () => {
  if (server?.listening) {
    await new Promise<void>((resolve, reject) => {
      server.close(error => error ? reject(error) : resolve())
    })
  }

  if (databaseModule) {
    // Queue behind fire-and-forget visitor tracking before closing the handle.
    await databaseModule.duckdb.query('CHECKPOINT')
    await databaseModule.closeDatabases()
  }

  if (tempDirectory) {
    await rm(tempDirectory, { recursive: true, force: true })
  }
})

describe('backend API characterization', () => {
  it('logs in a seeded administrator and rejects a wrong password', async () => {
    const valid = await postJson('/api/auth/login', {
      email: 'admin@hesocial.com',
      password: 'admin123',
    })
    expect(valid.response.status).toBe(200)
    expect(valid.body).toMatchObject({
      success: true,
      data: { token: expect.any(String) },
    })

    const invalid = await postJson('/api/auth/login', {
      email: 'admin@hesocial.com',
      password: 'not-the-password',
    })
    expect(invalid.response.status).toBe(401)
  })

  it('registers a new user', async () => {
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

  it('returns the current registration failure for a missing password', async () => {
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
    const result = await request('/api/events')

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
    const unauthenticated = await request('/api/admin/database/stats')
    expect([401, 403]).toContain(unauthenticated.response.status)

    const login = await postJson('/api/auth/login', {
      email: 'admin@hesocial.com',
      password: 'admin123',
    })
    const token = login.body.data.token as string
    const authenticated = await request('/api/admin/database/stats', {
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
