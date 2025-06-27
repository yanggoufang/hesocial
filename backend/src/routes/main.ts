import { Router } from 'express'

const createRoutes = async (): Promise<Router> => {
  const { default: routes } = await import('./index.js')
  return routes
}

export default createRoutes