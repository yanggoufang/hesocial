const configuredApiUrl = import.meta.env.VITE_API_URL || 'http://localhost:5000'
const normalizedApiUrl = configuredApiUrl.replace(/\/+$/, '')

export const API_BASE_URL = normalizedApiUrl.endsWith('/api')
  ? normalizedApiUrl
  : `${normalizedApiUrl}/api`
