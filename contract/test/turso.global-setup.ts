import { spawn, type ChildProcess } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const contractDirectory = dirname(fileURLToPath(import.meta.url))
const rustDirectory = resolve(contractDirectory, '../../backend-rust')

export const TURSO_TEST_PORT = Number(process.env.TURSO_TEST_PORT ?? 8481)
const endpoint = `http://127.0.0.1:${TURSO_TEST_PORT}/v2/pipeline`

/** `turso dev` is installed outside PATH by the official installer. */
const resolveTursoBinary = (): string => {
  const configured = process.env.TURSO_BIN
  if (configured) {
    return configured
  }
  const installed = resolve(homedir(), '.turso/turso')
  return existsSync(installed) ? installed : 'turso'
}

/**
 * Mirrors the splitting the D1 harness did: the schema/seed files are plain
 * scripts, and sqld's pipeline takes one statement per request.
 */
const splitStatements = (sql: string): string[] =>
  sql
    .split('\n')
    .filter((line) => {
      const trimmed = line.trimStart()
      return !trimmed.startsWith('--') && !trimmed.toUpperCase().startsWith('PRAGMA')
    })
    .join('\n')
    .split(';')
    .map((statement) => statement.trim())
    .filter((statement) => statement.length > 0)

const pipeline = async (requests: unknown[]): Promise<unknown[]> => {
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ requests }),
  })
  if (!response.ok) {
    throw new Error(`turso dev returned HTTP ${response.status}`)
  }
  const payload = (await response.json()) as { results?: unknown[] }
  return payload.results ?? []
}

const waitForReady = async (server: ChildProcess): Promise<void> => {
  const deadline = Date.now() + 20_000
  let lastError: unknown
  while (Date.now() < deadline) {
    if (server.exitCode !== null) {
      throw new Error(`turso dev exited early with code ${server.exitCode}`)
    }
    try {
      await pipeline([{ type: 'close' }])
      return
    } catch (error) {
      lastError = error
      await new Promise((sleep) => setTimeout(sleep, 150))
    }
  }
  throw new Error(`turso dev never became ready on port ${TURSO_TEST_PORT}: ${String(lastError)}`)
}

const applyScript = async (path: string): Promise<void> => {
  const statements = splitStatements(readFileSync(path, 'utf8'))
  const results = await pipeline([
    ...statements.map((sql) => ({ type: 'execute', stmt: { sql } })),
    { type: 'close' },
  ])
  results.forEach((result, index) => {
    const step = result as { type?: string; error?: { message?: string } }
    if (step.type !== 'ok') {
      throw new Error(`${path} statement ${index + 1} failed: ${step.error?.message ?? 'unknown'}\n${statements[index]}`)
    }
  })
}

export default async function setup(): Promise<() => Promise<void>> {
  // No `--db-file`, so each run gets a throwaway in-memory database — the
  // isolation miniflare's local D1 used to provide.
  const server = spawn(resolveTursoBinary(), ['dev', '--port', String(TURSO_TEST_PORT)], {
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  let output = ''
  server.stdout?.on('data', (chunk) => {
    output += String(chunk)
  })
  server.stderr?.on('data', (chunk) => {
    output += String(chunk)
  })
  server.on('error', (error) => {
    output += `\n${error.message}`
  })

  try {
    await waitForReady(server)
    await applyScript(resolve(rustDirectory, 'sql/schema.sql'))
    await applyScript(resolve(rustDirectory, 'sql/seed.sql'))
  } catch (error) {
    server.kill('SIGKILL')
    throw new Error(`${String(error)}\n--- turso dev output ---\n${output}`)
  }

  return async () => {
    server.kill('SIGTERM')
  }
}
