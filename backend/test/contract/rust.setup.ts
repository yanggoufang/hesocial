import { env } from 'cloudflare:workers'
import { beforeAll } from 'vitest'

interface RustContractEnv {
  DB: D1Database
  TEST_SCHEMA_SQL: string
  TEST_SEED_SQL: string
}

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

beforeAll(async () => {
  const bindings = env as RustContractEnv
  const statements = [
    ...splitStatements(bindings.TEST_SCHEMA_SQL),
    ...splitStatements(bindings.TEST_SEED_SQL),
  ]
  for (const statement of statements) {
    await bindings.DB.prepare(statement).run()
  }
})
