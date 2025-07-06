import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api'

export interface MediaFile {
  id: string
  type: 'image' | 'document'
  filePath: string
  thumbnails?: Record<string, string>
  originalFilename: string
  fileSize: number
  mimeType: string
  uploadedBy: string
  createdAt: string
}

export interface MediaUploadResult {
  eventId?: string
  venueId?: string
  uploadedImages?: MediaFile[]
  uploadedDocuments?: MediaFile[]
  count: number
}

export interface MediaStats {
  eventMedia: {
    total: number
    images: number
    documents: number
    totalSize: number
  }
  venueMedia: {
    total: number
    totalSize: number
  }
  recentUploads: number
  totalFiles: number
  totalStorageUsed: number
}

class MediaService {
  private baseURL: string

  constructor() {
    this.baseURL = `${API_BASE_URL}/media`
  }

  /**
   * Upload images for an event
   */
  async uploadEventImages(eventId: string, files: FileList): Promise<MediaUploadResult> {
    try {
      const formData = new FormData()
      
      Array.from(files).forEach(file => {
        formData.append('eventImages', file)
      })

      const response = await axios.post(
        `${this.baseURL}/events/${eventId}/images`,
        formData,
        {
          headers: {
            'Content-Type': 'multipart/form-data'
          }
        }
      )

      return response.data.data
    } catch (error: any) {
      console.error('Upload event images error:', error)
      throw new Error(error.response?.data?.error || 'Failed to upload images')
    }
  }

  /**
   * Upload documents for an event
   */
  async uploadEventDocuments(eventId: string, files: FileList): Promise<MediaUploadResult> {
    try {
      const formData = new FormData()
      
      Array.from(files).forEach(file => {
        formData.append('eventDocuments', file)
      })

      const response = await axios.post(
        `${this.baseURL}/events/${eventId}/documents`,
        formData,
        {
          headers: {
            'Content-Type': 'multipart/form-data'
          }
        }
      )

      return response.data.data
    } catch (error: any) {
      console.error('Upload event documents error:', error)
      throw new Error(error.response?.data?.error || 'Failed to upload documents')
    }
  }

  /**
   * Get all media for an event
   */
  async getEventMedia(eventId: string, type?: 'image' | 'document'): Promise<MediaFile[]> {
    try {
      const params = type ? { type } : {}
      const response = await axios.get(`${this.baseURL}/events/${eventId}`, { params })
      return response.data.data
    } catch (error: any) {
      console.error('Get event media error:', error)
      throw new Error(error.response?.data?.error || 'Failed to get event media')
    }
  }

  /**
   * Upload images for a venue
   */
  async uploadVenueImages(venueId: string, files: FileList): Promise<MediaUploadResult> {
    try {
      const formData = new FormData()
      
      Array.from(files).forEach(file => {
        formData.append('venueImages', file)
      })

      const response = await axios.post(
        `${this.baseURL}/venues/${venueId}/images`,
        formData,
        {
          headers: {
            'Content-Type': 'multipart/form-data'
          }
        }
      )

      return response.data.data
    } catch (error: any) {
      console.error('Upload venue images error:', error)
      throw new Error(error.response?.data?.error || 'Failed to upload venue images')
    }
  }

  /**
   * Get all media for a venue
   */
  async getVenueMedia(venueId: string): Promise<MediaFile[]> {
    try {
      const response = await axios.get(`${this.baseURL}/venues/${venueId}`)
      return response.data.data
    } catch (error: any) {
      console.error('Get venue media error:', error)
      throw new Error(error.response?.data?.error || 'Failed to get venue media')
    }
  }

  /**
   * Delete a media file
   */
  async deleteMedia(mediaId: string): Promise<void> {
    try {
      await axios.delete(`${this.baseURL}/${mediaId}`)
    } catch (error: any) {
      console.error('Delete media error:', error)
      throw new Error(error.response?.data?.error || 'Failed to delete media')
    }
  }

  /**
   * Download a document (get signed URL)
   */
  async downloadDocument(mediaId: string): Promise<{ downloadUrl: string; filename: string; expiresIn: number }> {
    try {
      const response = await axios.get(`${this.baseURL}/download/${mediaId}`)
      return response.data.data
    } catch (error: any) {
      console.error('Download document error:', error)
      throw new Error(error.response?.data?.error || 'Failed to generate download URL')
    }
  }

  /**
   * Get media statistics (admin only)
   */
  async getMediaStats(): Promise<MediaStats> {
    try {
      const response = await axios.get(`${this.baseURL}/stats`)
      return response.data.data
    } catch (error: any) {
      console.error('Get media stats error:', error)
      throw new Error(error.response?.data?.error || 'Failed to get media statistics')
    }
  }

  /**
   * Clean up orphaned media files (super admin only)
   */
  async cleanupOrphanedMedia(): Promise<{
    deletedFiles: number
    deletedRecords: number
    orphanedEventMedia: number
    orphanedVenueMedia: number
  }> {
    try {
      const response = await axios.post(`${this.baseURL}/cleanup`)
      return response.data.data
    } catch (error: any) {
      console.error('Cleanup media error:', error)
      throw new Error(error.response?.data?.error || 'Failed to cleanup media')
    }
  }

  /**
   * Format file size for display
   */
  formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B'
    
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  /**
   * Get file type icon
   */
  getFileTypeIcon(mimeType: string): string {
    if (mimeType.startsWith('image/')) {
      return '🖼️'
    } else if (mimeType === 'application/pdf') {
      return '📄'
    } else if (mimeType.includes('word')) {
      return '📝'
    } else if (mimeType.includes('excel') || mimeType.includes('spreadsheet')) {
      return '📊'
    } else {
      return '📎'
    }
  }

  /**
   * Validate file for upload
   */
  validateFile(file: File, maxSizeMB: number = 10): { valid: boolean; error?: string } {
    // Check file size
    if (file.size > maxSizeMB * 1024 * 1024) {
      return { valid: false, error: `File size must be less than ${maxSizeMB}MB` }
    }

    // Check file type
    const allowedTypes = [
      'image/jpeg',
      'image/png', 
      'image/webp',
      'image/gif',
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'application/vnd.ms-excel',
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
    ]

    if (!allowedTypes.includes(file.type)) {
      return { valid: false, error: 'File type not supported. Please upload images or documents only.' }
    }

    return { valid: true }
  }

  /**
   * Create thumbnail URL from original image URL
   */
  getThumbnailUrl(filePath: string, size: 'thumb' | 'medium' | 'large' = 'thumb'): string {
    // For R2 stored images, we have pre-generated thumbnails
    // This is just a fallback - actual thumbnails are returned in the thumbnails object
    return filePath
  }
}

export const mediaService = new MediaService()