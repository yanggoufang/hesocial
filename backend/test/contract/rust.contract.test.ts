import { exports as workerExports } from 'cloudflare:workers'
import {
  defineContractTests,
  SEEDED_ADMIN_CREDENTIALS,
  type ContractRequest,
} from './api.contract.js'

const request: ContractRequest = async (path, init) => {
  const response = await workerExports.default.fetch(`http://example.com${path}`, init)
  const body = await response.json()
  return { body, response: { status: response.status } }
}

defineContractTests({
  request,
  seededCredentials: SEEDED_ADMIN_CREDENTIALS,
  authImplemented: true,
  adminStatsExpectation: 'unauthorized',
  eventsImplemented: true,
  registrationsImplemented: true,
  participantsImplemented: true,
  salesImplemented: true,
  salesFlowImplemented: true,
})
