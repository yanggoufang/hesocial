import { existsSync, readFileSync } from 'node:fs'
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
