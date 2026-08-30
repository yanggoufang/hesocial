import { tmpdir } from 'node:os'
import { join } from 'node:path'

process.env.NODE_ENV = 'test'
process.env.AUTH_RATE_LIMIT_MAX = '1000'
process.env.RATE_LIMIT_MAX_REQUESTS = '1000'
process.env.DUCKDB_PATH = join(tmpdir(), 'hesocial-vitest-throwaway.duckdb')
