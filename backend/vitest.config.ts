import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    setupFiles: ['./test/setup.ts'],
    hookTimeout: 30_000,
    testTimeout: 15_000,
    sequence: {
      concurrent: false,
    },
  },
})
