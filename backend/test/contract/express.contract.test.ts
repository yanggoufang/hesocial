import { IncomingMessage, ServerResponse, type Server } from 'node:http'
import { mkdtemp, readFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { Duplex } from 'node:stream'
import { fileURLToPath } from 'node:url'
import type { Express } from 'express'
import { afterAll, beforeAll } from 'vitest'
import {
  defineContractTests,
  SEEDED_ADMIN_CREDENTIALS,
  type ContractRequest,
} from './api.contract.js'

let server: Server | undefined
let baseUrl: string | undefined
let expressApp: Express
let tempDirectory: string
let databaseModule: typeof import('../../src/database/duckdb-connection.js')

const requestInProcess: ContractRequest = async (path, init) => {
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

const request: ContractRequest = async (path, init) => {
  if (baseUrl) {
    const response = await fetch(`${baseUrl}${path}`, init)
    const body = await response.json()
    return { body, response: { status: response.status } }
  }

  return requestInProcess(path, init)
}

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

  databaseModule = await import('../../src/database/duckdb-connection.js')

  const schemaPath = fileURLToPath(new URL('../../../database/duckdb-schema.sql', import.meta.url))
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


  // Phase 2f sales CRM fixture. DuckDB has no autoincrement for
  // `id INTEGER PRIMARY KEY`, so every seeded row carries an explicit id — the
  // same ids the D1 seed uses. DuckDB also predates several D1 columns
  // (last_contact_date, actual_close_date, close_reason, color_code,
  // sales_activities.updated_at), so those are simply absent here.
  await databaseModule.duckdb.query(`
    INSERT OR IGNORE INTO sales_leads (
      id, first_name, last_name, email, phone, company, job_title,
      annual_income, net_worth, source, referral_code, lead_score, status,
      interested_membership_tier, budget_range, timeline, pain_points,
      interests, notes, assigned_to, created_at, updated_at
    ) VALUES
      (9001, 'Seeded', 'Contract', 'crm-active@hesocial.test', '+886900000001',
       'Contract Holdings', 'Principal', 25000000, 120000000, 'referral', 'CRM2F',
       100, 'new', 'Black Card', '5-10M', 'this-quarter', 'Discreet networking',
       '["fine dining","yachting"]', 'Active contract lead',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9002, 'Legacy', 'Won', 'crm-won@hesocial.test', '+886900000002',
       'Founding Member Co', 'Chair', 40000000, 200000000, 'event', NULL,
       100, 'closed_won', 'Black Card', '10M+', 'closed', 'Privacy',
       '["art", "yachting"]', 'Historical closed-won lead outside the window',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       '2020-01-01 00:00:00', '2020-01-20 00:00:00'),
      (9003, 'Deletable', 'Row', 'crm-deletable@hesocial.test', NULL, NULL, NULL,
       NULL, NULL, 'website', NULL, 0, 'new', NULL, NULL, NULL, NULL,
       '[]', 'Childless row reserved for the delete route', NULL,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9004, 'Updatable', 'Target', 'crm-updatable@hesocial.test', '+886900000004',
       'Renewal Co', 'Director', 8000000, 40000000, 'website', NULL,
       40, 'contacted', 'Platinum', '1-5M', 'next-quarter', 'Sparse network',
       '["networking"]', 'Childless row reserved for the update route',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

    INSERT OR IGNORE INTO sales_opportunities (
      id, lead_id, name, description, stage, probability, value,
      expected_close_date, membership_tier, payment_terms,
      assigned_to, created_at, updated_at
    ) VALUES
      (9101, 9002, 'Legacy Black Card Founding Seat',
       'Closed-won historical deal outside the reporting window',
       'closed_won', 100, 250000, '2020-02-01', 'Black Card', 'annual-prepaid',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       '2020-01-01 00:00:00', '2020-01-20 00:00:00'),
      (9102, 9001, 'Diamond Membership Renewal',
       'Open deal the contract reads through the stage filter',
       'proposal', 60, 480000, '2026-12-01', 'Diamond', 'semi-annual',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9103, 9002, 'Negotiation Seat (childless)',
       'Open deal with no logged activity, reserved for the stage-transition test',
       'negotiation', 80, 120000, '2026-10-31', 'Platinum', 'one-time',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

    INSERT OR IGNORE INTO sales_activities (
      id, lead_id, opportunity_id, activity_type, subject, description, outcome,
      duration_minutes, scheduled_at, completed_at, created_by, created_at
    ) VALUES
      (9201, 9002, 9101, 'meeting', 'Founding seat presentation',
       'Historical close meeting', 'signed', 60,
       '2020-01-15 02:00:00', '2020-01-15 03:00:00',
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       '2020-01-15 03:00:00'),
      (9202, 9001, 9102, 'call', 'Renewal discovery call',
       'Confirmed the Diamond tier budget', 'reached', 30,
       NULL, NULL,
       (SELECT id FROM users WHERE email = 'admin@hesocial.com'),
       CURRENT_TIMESTAMP);

    INSERT OR IGNORE INTO sales_pipeline_stages (
      id, name, description, display_order, default_probability, is_active,
      created_at, updated_at
    ) VALUES
      (9401, 'qualification', 'Identify and validate the prospect', 1, 25, true,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9402, 'needs_analysis', 'Document tier expectations and budget', 2, 40, true,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9403, 'proposal', 'Present the membership proposal', 3, 60, true,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9404, 'negotiation', 'Negotiate terms and the close date', 4, 80, true,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9405, 'archived_legacy', 'Retired stage that must stay hidden', 9, 0, false,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

    INSERT OR IGNORE INTO sales_team_members (
      id, user_id, role, territory, commission_rate, quota_amount, is_active,
      hire_date, manager_id, created_at, updated_at
    ) VALUES
      (9301, (SELECT id FROM users WHERE email = 'admin@hesocial.com'), 'sales_rep',
       'Taipei', 8.50, 3000000, true, '2024-03-01',
       (SELECT id FROM users WHERE email = 'test.platinum@example.com'),
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
      (9302, (SELECT id FROM users WHERE email = 'test.platinum@example.com'),
       'sales_manager', 'Kaohsiung', 12.00, 9000000, false, '2022-06-01', NULL,
       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
  `)

  const [{ default: app }, { default: createRoutes }] = await Promise.all([
    import('../../src/server.js'),
    import('../../src/routes/main.js'),
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

defineContractTests({
  request,
  seededCredentials: SEEDED_ADMIN_CREDENTIALS,
  // Express has routes, but not the Phase 2d semantics: capacity returns 400,
  // writes are non-transactional, and there is no waitlist promotion. Keeping
  // this false also preserves the characterization target at 6 pass/7 skip.
  registrationsImplemented: false,
  // The temp DB has the three participant tables, but it does not reproduce
  // the live handler's required user privacy columns or the registration_id
  // access join. The Express list also maps the DuckDB wrapper rather than
  // result.rows, so seeded dual-target participant tests would currently 500.
  participantsImplemented: false,
  // The sales routes are mounted live on Express and the characterization temp
  // DB creates all five sales tables, so the Phase 2f read/filter/update/delete/
  // metrics/pipeline/team assertions run against the mirrored fixture on both
  // targets. The follow-up block stays off here: Express cannot INSERT a sales
  // row at all (DuckDB gives `id INTEGER PRIMARY KEY` no sequence), its leads
  // `search` and opportunities `membershipTier` filters 500 on ambiguous column
  // references, and updating a lead that owns an opportunity 500s on the child
  // foreign key. See the salesFlowImplemented block in api.contract.ts.
  salesImplemented: true,
})
