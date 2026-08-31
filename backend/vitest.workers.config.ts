import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { cloudflareTest } from '@cloudflare/vitest-pool-workers'
import { defineConfig } from 'vitest/config'

const backendDirectory = dirname(fileURLToPath(import.meta.url))
const rustDirectory = resolve(backendDirectory, '../backend-rust')
const wranglerConfigPath = resolve(rustDirectory, 'wrangler.test.toml')
const workerShimPath = resolve(rustDirectory, 'crates/worker/build/worker/shim.mjs')

if (!existsSync(workerShimPath)) {
  throw new Error(
    `Rust worker build is missing at ${workerShimPath}. `
    + 'Run `cd backend-rust && npx wrangler deploy --dry-run` before the Rust contract tests.',
  )
}

const newestMtime = (path: string): number => {
  const stats = statSync(path)
  if (!stats.isDirectory()) {
    return stats.mtimeMs
  }
  return readdirSync(path, { withFileTypes: true }).reduce((newest, entry) => {
    const mtime = newestMtime(resolve(path, entry.name))
    return Math.max(newest, mtime)
  }, stats.mtimeMs)
}

const newestRustSourceMtime = [
  'crates/core/src',
  'crates/worker/src',
  'd1',
  'Cargo.toml',
  'Cargo.lock',
  'wrangler.toml',
  'wrangler.test.toml',
]
  .map((relative) => newestMtime(resolve(rustDirectory, relative)))
  .reduce((newest, current) => Math.max(newest, current), 0)
const shimMtime = statSync(workerShimPath).mtimeMs

if (shimMtime < newestRustSourceMtime) {
  throw new Error(
    `Rust worker build is stale at ${workerShimPath} (older than the Rust sources). `
    + 'Run `cd backend-rust && npx wrangler deploy --dry-run` to rebuild, then re-run the Rust contract tests.',
  )
}

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: wranglerConfigPath },
      miniflare: {
        bindings: {
          TEST_SCHEMA_SQL: readFileSync(resolve(rustDirectory, 'd1/schema.sql'), 'utf8'),
          TEST_SEED_SQL: readFileSync(resolve(rustDirectory, 'd1/seed.sql'), 'utf8'),
        },
      },
    }),
  ],
  test: {
    include: ['test/contract/rust.contract.test.ts'],
    setupFiles: ['./test/contract/rust.setup.ts'],
    hookTimeout: 30_000,
    testTimeout: 15_000,
    fileParallelism: false,
  },
})
