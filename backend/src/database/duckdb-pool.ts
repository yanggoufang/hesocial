// DuckDB Pool Adapter
// This file provides a DuckDB interface
// so we can use existing controllers with minimal changes

import { duckdb } from './duckdb-connection.js'

export const pool = {
  async query(sql: string, params: any[] = []) {
    // Convert PostgreSQL-style $1, $2 parameters to DuckDB format
    let duckdbSQL = sql
    if (params && params.length > 0) {
      // Replace $1, $2, etc. with ? placeholders
      for (let i = params.length; i >= 1; i--) {
        duckdbSQL = duckdbSQL.replace(new RegExp(`\\$${i}`, 'g'), '?')
      }
    }

    return await duckdb.query(duckdbSQL, params)
  },

  async connect() {
    // Already connected through duckdb connection
    return Promise.resolve()
  },

  async end() {
    return await duckdb.close()
  }
}

// Mock Redis client for development
export const redisClient = {
  connect: async () => console.log('Redis mock: connected'),
  quit: async () => console.log('Redis mock: disconnected'),
  on: (event: string, callback: Function) => console.log(`Redis mock: listening to ${event}`)
}