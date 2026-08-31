import { describe, expect, it } from 'vitest'
import type { ContractRequest, SeededCredentials } from './api.contract.js'

export interface MediaContractRunner {
  request: ContractRequest
  seededCredentials: SeededCredentials
  mediaImplemented?: boolean
}

const uploadBody = (field: string, filename: string, mime: string, bytes: number[]): FormData => {
  const form = new FormData()
  form.append(field, new Blob([new Uint8Array(bytes)], { type: mime }), filename)
  return form
}

export const defineMediaContractTests = (runner: MediaContractRunner): void => {
  const mediaTest = it.skipIf(runner.mediaImplemented !== true)
  const postJson = (path: string, body: unknown) => runner.request(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  })
  const tokenFor = async (credentials: SeededCredentials) => {
    const login = await postJson('/api/auth/login', credentials)
    expect(login.response.status).toBe(200)
    return login.body.data.token as string
  }
  const auth = (token: string) => ({ authorization: `Bearer ${token}` })

  describe('media and R2 (Phase 2h)', () => {
    mediaTest('uploads, lists, authorizes, and deletes event/venue media', async () => {
      const anonymous = await runner.request('/api/media/events/2/images', {
        method: 'POST',
        body: uploadBody('eventImages', 'anonymous.png', 'image/png', [1, 2, 3]),
      })
      expect(anonymous.response.status).toBe(401)
      expect(anonymous.body).toEqual({ success: false, error: 'Access token required' })

      const memberToken = await tokenFor({
        email: 'test.platinum@example.com',
        password: 'test123',
      })
      const forbidden = await runner.request('/api/media/events/2/images', {
        method: 'POST',
        headers: auth(memberToken),
        body: uploadBody('eventImages', 'member.png', 'image/png', [1, 2, 3]),
      })
      expect(forbidden.response.status).toBe(403)
      expect(forbidden.body).toEqual({ success: false, error: 'Permission denied' })

      const adminToken = await tokenFor(runner.seededCredentials)
      const imageUpload = await runner.request('/api/media/events/2/images', {
        method: 'POST',
        headers: auth(adminToken),
        body: uploadBody('eventImages', 'Contract Photo.PNG', 'image/png', [137, 80, 78, 71]),
      })
      expect(imageUpload.response.status).toBe(200)
      expect(imageUpload.body).toMatchObject({
        success: true,
        data: {
          eventId: '2',
          count: 1,
          uploadedImages: [{
            id: expect.any(String),
            type: 'image',
            filePath: expect.stringMatching(/^https:\/\/media\.hesocial\.test\/events\//),
            thumbnails: {
              thumb: expect.any(String),
              medium: expect.any(String),
            },
            originalFilename: 'Contract Photo.PNG',
            fileSize: 4,
            mimeType: 'image/png',
          }],
        },
      })
      const imageId = imageUpload.body.data.uploadedImages[0].id as string

      const documentUpload = await runner.request('/api/media/events/2/documents', {
        method: 'POST',
        headers: auth(adminToken),
        body: uploadBody('eventDocuments', 'contract.pdf', 'application/pdf', [37, 80, 68, 70]),
      })
      expect(documentUpload.response.status).toBe(200)
      expect(documentUpload.body).toMatchObject({
        success: true,
        data: {
          eventId: '2',
          count: 1,
          uploadedDocuments: [{
            id: expect.any(String),
            type: 'document',
            filePath: expect.stringMatching(/^events\//),
            originalFilename: 'contract.pdf',
            fileSize: 4,
            mimeType: 'application/pdf',
          }],
        },
      })
      const documentId = documentUpload.body.data.uploadedDocuments[0].id as string

      const eventList = await runner.request('/api/media/events/2')
      expect(eventList.response.status).toBe(200)
      expect(eventList.body.data).toEqual(expect.arrayContaining([
        expect.objectContaining({
          id: imageId,
          eventId: 2,
          type: 'image',
          thumbnailPath: {
            thumb: expect.any(String),
            medium: expect.any(String),
          },
        }),
        expect.objectContaining({
          id: documentId,
          eventId: 2,
          type: 'document',
          filePath: null,
          thumbnailPath: null,
        }),
      ]))
      const documentsOnly = await runner.request('/api/media/events/2?type=document')
      expect(documentsOnly.body.data.map((row: any) => row.id)).toContain(documentId)
      expect(documentsOnly.body.data.every((row: any) => row.type === 'document')).toBe(true)

      const venueUpload = await runner.request('/api/media/venues/2/images', {
        method: 'POST',
        headers: auth(adminToken),
        body: uploadBody('venueImages', 'Yacht.webp', 'image/webp', [82, 73, 70, 70]),
      })
      expect(venueUpload.response.status).toBe(200)
      expect(venueUpload.body).toMatchObject({
        success: true,
        data: {
          venueId: '2',
          count: 1,
          uploadedImages: [{
            id: expect.any(String),
            thumbnails: {
              thumb: expect.any(String),
              medium: expect.any(String),
              large: expect.any(String),
            },
          }],
        },
      })
      const venueMediaId = venueUpload.body.data.uploadedImages[0].id as string
      const venueList = await runner.request('/api/media/venues/2')
      expect(venueList.response.status).toBe(200)
      expect(venueList.body.data).toEqual(expect.arrayContaining([
        expect.objectContaining({ id: venueMediaId, venueId: 2, type: 'image' }),
      ]))

      const invalidMime = await runner.request('/api/media/events/2/images', {
        method: 'POST',
        headers: auth(adminToken),
        body: uploadBody('eventImages', 'notes.txt', 'text/plain', [110, 111]),
      })
      expect(invalidMime.response.status).toBe(500)
      expect(invalidMime.body).toEqual({ success: false, error: 'Failed to upload images' })

      for (const id of [imageId, documentId, venueMediaId]) {
        const removed = await runner.request(`/api/media/${id}`, {
          method: 'DELETE',
          headers: auth(adminToken),
        })
        expect(removed.response.status).toBe(200)
        expect(removed.body).toEqual({ success: true, message: 'Media deleted successfully' })
      }
      const missing = await runner.request(`/api/media/${imageId}`, {
        method: 'DELETE',
        headers: auth(adminToken),
      })
      expect(missing.response.status).toBe(404)
      expect(missing.body).toEqual({ success: false, error: 'Media not found' })
    })
  })
}
