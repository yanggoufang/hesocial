import { S3Client, PutObjectCommand, DeleteObjectCommand, GetObjectCommand } from '@aws-sdk/client-s3'
import sharp from 'sharp'
import { v4 as uuidv4 } from 'uuid'
import path from 'path'
import { duckdb } from '../database/duckdb-connection.js'
import logger from '../utils/logger.js'
import config from '../utils/config.js'

export interface MediaUploadResult {
  id: string
  type: 'image' | 'document'
  filePath: string
  thumbnails?: Record<string, string>
  originalFilename: string
  fileSize: number
  mimeType: string
}

export interface ThumbnailSize {
  width: number
  height: number
  suffix: string
}

class MediaService {
  private r2Client: S3Client
  private bucketName: string

  constructor() {
    this.r2Client = new S3Client({
      region: 'auto',
      endpoint: config.r2.endpoint,
      credentials: {
        accessKeyId: config.r2.accessKeyId,
        secretAccessKey: config.r2.secretAccessKey,
      }
    })
    this.bucketName = config.r2.bucketName
  }

  /**
   * Generate unique filename with proper extension
   */
  private generateFileName(originalName: string, category: string, suffix?: string): string {
    const ext = path.extname(originalName)
    const baseName = path.basename(originalName, ext)
    const uniqueId = uuidv4()
    const safeName = baseName.replace(/[^a-zA-Z0-9]/g, '-').toLowerCase()
    
    if (suffix) {
      return `${category}/${safeName}-${uniqueId}-${suffix}${ext}`
    }
    return `${category}/${safeName}-${uniqueId}${ext}`
  }

  /**
   * Upload file to R2 storage
   */
  private async uploadToR2(
    buffer: Buffer, 
    key: string, 
    contentType: string,
    metadata?: Record<string, string>
  ): Promise<string> {
    try {
      const command = new PutObjectCommand({
        Bucket: this.bucketName,
        Key: key,
        Body: buffer,
        ContentType: contentType,
        Metadata: metadata,
        // Make images publicly accessible
        ACL: contentType.startsWith('image/') ? 'public-read' : 'private'
      })

      await this.r2Client.send(command)
      
      // Return public URL for images, signed URL for documents
      if (contentType.startsWith('image/')) {
        return `${config.r2.publicUrl}/${key}`
      } else {
        // For documents, return the key - we'll generate signed URLs when needed
        return key
      }
    } catch (error) {
      logger.error('R2 upload error:', error)
      throw new Error('Failed to upload file to storage')
    }
  }

  /**
   * Delete file from R2 storage
   */
  private async deleteFromR2(key: string): Promise<void> {
    try {
      const command = new DeleteObjectCommand({
        Bucket: this.bucketName,
        Key: key
      })
      await this.r2Client.send(command)
    } catch (error) {
      logger.error('R2 delete error:', error)
      throw new Error('Failed to delete file from storage')
    }
  }

  /**
   * Process image and generate thumbnails
   */
  private async processImage(
    buffer: Buffer,
    originalName: string,
    category: string,
    sizes: ThumbnailSize[] = []
  ): Promise<{ original: string; thumbnails: Record<string, string> }> {
    try {
      // Default thumbnail sizes if none provided
      if (sizes.length === 0) {
        sizes = [
          { width: 400, height: 300, suffix: 'thumb' },
          { width: 800, height: 600, suffix: 'medium' }
        ]
      }

      // Process original image (max 1920x1080, optimized)
      const originalBuffer = await sharp(buffer)
        .resize(1920, 1080, { 
          fit: 'inside',
          withoutEnlargement: true 
        })
        .jpeg({ quality: 85 })
        .toBuffer()

      // Upload original
      const originalKey = this.generateFileName(originalName, category, 'original')
      const originalUrl = await this.uploadToR2(
        originalBuffer, 
        originalKey, 
        'image/jpeg',
        { 
          originalName,
          category,
          type: 'original'
        }
      )

      // Generate and upload thumbnails
      const thumbnails: Record<string, string> = {}
      
      for (const size of sizes) {
        const thumbBuffer = await sharp(buffer)
          .resize(size.width, size.height, { 
            fit: 'cover',
            position: 'center'
          })
          .jpeg({ quality: 80 })
          .toBuffer()

        const thumbKey = this.generateFileName(originalName, category, size.suffix)
        const thumbUrl = await this.uploadToR2(
          thumbBuffer, 
          thumbKey, 
          'image/jpeg',
          {
            originalName,
            category,
            type: 'thumbnail',
            size: `${size.width}x${size.height}`
          }
        )
        
        thumbnails[size.suffix] = thumbUrl
      }

      return {
        original: originalUrl,
        thumbnails
      }
    } catch (error) {
      logger.error('Image processing error:', error)
      throw new Error('Failed to process image')
    }
  }

  /**
   * Upload document to R2
   */
  private async uploadDocument(
    buffer: Buffer,
    originalName: string,
    mimeType: string,
    category: string
  ): Promise<string> {
    try {
      const key = this.generateFileName(originalName, category)
      
      // Upload document (private by default)
      const documentKey = await this.uploadToR2(
        buffer, 
        key, 
        mimeType,
        {
          originalName,
          category,
          type: 'document'
        }
      )

      return documentKey
    } catch (error) {
      logger.error('Document upload error:', error)
      throw new Error('Failed to upload document')
    }
  }

  /**
   * Upload event images
   */
  async uploadEventImages(
    eventId: string,
    files: Express.Multer.File[],
    uploadedBy: string
  ): Promise<MediaUploadResult[]> {
    const results: MediaUploadResult[] = []

    for (const file of files) {
      try {
        const imageData = await this.processImage(
          file.buffer,
          file.originalname,
          'events'
        )

        // Save to database
        const mediaId = uuidv4()
        await duckdb.prepare(`
          INSERT INTO event_media (
            id, event_id, type, file_path, thumbnail_path, 
            original_filename, file_size, mime_type, uploaded_by, created_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        `).run(
          mediaId,
          eventId,
          'image',
          imageData.original,
          JSON.stringify(imageData.thumbnails),
          file.originalname,
          file.size,
          file.mimetype,
          uploadedBy
        )

        results.push({
          id: mediaId,
          type: 'image',
          filePath: imageData.original,
          thumbnails: imageData.thumbnails,
          originalFilename: file.originalname,
          fileSize: file.size,
          mimeType: file.mimetype
        })
      } catch (error) {
        logger.error(`Error processing event image ${file.originalname}:`, error)
      }
    }

    return results
  }

  /**
   * Upload event documents
   */
  async uploadEventDocuments(
    eventId: string,
    files: Express.Multer.File[],
    uploadedBy: string
  ): Promise<MediaUploadResult[]> {
    const results: MediaUploadResult[] = []

    for (const file of files) {
      try {
        const documentPath = await this.uploadDocument(
          file.buffer,
          file.originalname,
          file.mimetype,
          'events'
        )

        // Save to database
        const mediaId = uuidv4()
        await duckdb.prepare(`
          INSERT INTO event_media (
            id, event_id, type, file_path, original_filename, 
            file_size, mime_type, uploaded_by, created_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        `).run(
          mediaId,
          eventId,
          'document',
          documentPath,
          file.originalname,
          file.size,
          file.mimetype,
          uploadedBy
        )

        results.push({
          id: mediaId,
          type: 'document',
          filePath: documentPath,
          originalFilename: file.originalname,
          fileSize: file.size,
          mimeType: file.mimetype
        })
      } catch (error) {
        logger.error(`Error processing event document ${file.originalname}:`, error)
      }
    }

    return results
  }

  /**
   * Upload venue images
   */
  async uploadVenueImages(
    venueId: string,
    files: Express.Multer.File[],
    uploadedBy: string
  ): Promise<MediaUploadResult[]> {
    const results: MediaUploadResult[] = []

    for (const file of files) {
      try {
        const imageData = await this.processImage(
          file.buffer,
          file.originalname,
          'venues',
          [
            { width: 300, height: 200, suffix: 'thumb' },
            { width: 600, height: 400, suffix: 'medium' },
            { width: 1200, height: 800, suffix: 'large' }
          ]
        )

        // Save to database
        const mediaId = uuidv4()
        await duckdb.prepare(`
          INSERT INTO venue_media (
            id, venue_id, type, file_path, thumbnail_path, 
            original_filename, file_size, mime_type, uploaded_by, created_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        `).run(
          mediaId,
          venueId,
          'image',
          imageData.original,
          JSON.stringify(imageData.thumbnails),
          file.originalname,
          file.size,
          file.mimetype,
          uploadedBy
        )

        results.push({
          id: mediaId,
          type: 'image',
          filePath: imageData.original,
          thumbnails: imageData.thumbnails,
          originalFilename: file.originalname,
          fileSize: file.size,
          mimeType: file.mimetype
        })
      } catch (error) {
        logger.error(`Error processing venue image ${file.originalname}:`, error)
      }
    }

    return results
  }

  /**
   * Delete media and its files from R2
   */
  async deleteMedia(mediaId: string): Promise<void> {
    try {
      // Get media record
      const media = await duckdb.prepare(`
        SELECT * FROM event_media WHERE id = ?
        UNION ALL
        SELECT * FROM venue_media WHERE id = ?
      `).get(mediaId, mediaId)

      if (!media) {
        throw new Error('Media not found')
      }

      // Extract R2 key from URL for images, or use direct key for documents
      let r2Key: string
      if (media.type === 'image' && media.file_path.startsWith('http')) {
        // Extract key from public URL
        const url = new URL(media.file_path)
        r2Key = url.pathname.substring(1) // Remove leading slash
      } else {
        // Document key is stored directly
        r2Key = media.file_path
      }

      // Delete main file
      await this.deleteFromR2(r2Key)

      // Delete thumbnails if they exist
      if (media.thumbnail_path) {
        try {
          const thumbnails = JSON.parse(media.thumbnail_path)
          for (const thumbnailUrl of Object.values(thumbnails) as string[]) {
            if (thumbnailUrl.startsWith('http')) {
              const url = new URL(thumbnailUrl)
              const thumbKey = url.pathname.substring(1)
              await this.deleteFromR2(thumbKey)
            }
          }
        } catch (error) {
          logger.warn('Error deleting thumbnails:', error)
        }
      }

      // Delete from database
      await duckdb.prepare(`
        DELETE FROM event_media WHERE id = ?
      `).run(mediaId)

      await duckdb.prepare(`
        DELETE FROM venue_media WHERE id = ?
      `).run(mediaId)

    } catch (error) {
      logger.error('Delete media error:', error)
      throw error
    }
  }

  /**
   * Generate signed URL for private documents
   */
  async getSignedUrl(documentKey: string, expiresIn: number = 3600): Promise<string> {
    try {
      const { getSignedUrl } = await import('@aws-sdk/s3-request-presigner')
      
      const command = new GetObjectCommand({
        Bucket: this.bucketName,
        Key: documentKey
      })

      const signedUrl = await getSignedUrl(this.r2Client, command, { 
        expiresIn 
      })

      return signedUrl
    } catch (error) {
      logger.error('Generate signed URL error:', error)
      throw new Error('Failed to generate download URL')
    }
  }

  /**
   * Get media statistics
   */
  async getMediaStats(): Promise<any> {
    try {
      const stats = await Promise.all([
        // Total event media
        duckdb.prepare(`
          SELECT 
            COUNT(*) as total,
            COUNT(CASE WHEN type = 'image' THEN 1 END) as images,
            COUNT(CASE WHEN type = 'document' THEN 1 END) as documents,
            COALESCE(SUM(file_size), 0) as totalSize
          FROM event_media
        `).get(),
        
        // Total venue media
        duckdb.prepare(`
          SELECT 
            COUNT(*) as total,
            COALESCE(SUM(file_size), 0) as totalSize
          FROM venue_media
        `).get(),
        
        // Recent uploads (last 30 days)
        duckdb.prepare(`
          SELECT COUNT(*) as recentUploads
          FROM event_media 
          WHERE created_at > datetime('now', '-30 days')
        `).get()
      ])

      const [eventStats, venueStats, recentStats] = stats

      return {
        eventMedia: {
          total: eventStats.total,
          images: eventStats.images,
          documents: eventStats.documents,
          totalSize: eventStats.totalSize
        },
        venueMedia: {
          total: venueStats.total,
          totalSize: venueStats.totalSize
        },
        recentUploads: recentStats.recentUploads,
        totalFiles: eventStats.total + venueStats.total,
        totalStorageUsed: eventStats.totalSize + venueStats.totalSize
      }
    } catch (error) {
      logger.error('Get media stats error:', error)
      throw error
    }
  }
}

export const mediaService = new MediaService()