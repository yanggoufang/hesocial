import { Router } from 'express'

const createUnifiedRoutes = async (): Promise<Router> => {
  const { default: routes } = await import('./index.js')
  return routes
}

export default createUnifiedRoutes