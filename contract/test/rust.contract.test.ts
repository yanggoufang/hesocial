import { exports as workerExports } from 'cloudflare:workers'
import {
  defineContractTests,
  SEEDED_ADMIN_CREDENTIALS,
  type ContractRequest,
} from './api.contract.js'
import { defineAnalyticsContractTests } from './analytics.contract.js'
import { defineMediaContractTests } from './media.contract.js'

const request: ContractRequest = async (path, init) => {
  const response = await workerExports.default.fetch(`http://example.com${path}`, init)
  const body = await response.json()
  return { body, response: { status: response.status } }
}

defineContractTests({
  request,
  seededCredentials: SEEDED_ADMIN_CREDENTIALS,
  authImplemented: true,
  // Phase 7: /api/admin/database/stats and /api/users/* are ported, so the
  // shared 401/200 auth flow and the user-management block run here too.
  adminStatsExpectation: 'authenticated',
  adminImplemented: true,
  adminListImplemented: true,
  eventsImplemented: true,
  registrationsImplemented: true,
  participantsImplemented: true,
  salesImplemented: true,
  salesFlowImplemented: true,
})

defineAnalyticsContractTests({
  request,
  seededCredentials: SEEDED_ADMIN_CREDENTIALS,
  analyticsImplemented: true,
  trackingRequiresAdmin: false,
})

defineMediaContractTests({
  request,
  seededCredentials: SEEDED_ADMIN_CREDENTIALS,
  mediaImplemented: true,
})
