import { Router } from 'express'
import { getDatabaseType } from '@/database/unified-connection.js'

const createUnifiedRoutes = async (): Promise<Router> => {
  const dbType = getDatabaseType()
  
  if (dbType === 'postgresql') {
    const { default: postgresRoutes } = await import('./index.js')
    return postgresRoutes
  } else {
    const { default: duckdbRoutes } = await import('./index-duckdb.js')
    return duckdbRoutes
  }
}

export default createUnifiedRoutes