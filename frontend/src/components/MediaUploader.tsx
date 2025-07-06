import { useState, useRef } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Upload, X, File, Image, AlertCircle, CheckCircle, Download, Trash2 } from 'lucide-react'
import { mediaService, MediaFile } from '../services/mediaService'

interface MediaUploaderProps {
  eventId?: string
  venueId?: string
  type: 'image' | 'document' | 'both'
  onUploadComplete?: (files: MediaFile[]) => void
  maxFiles?: number
  maxSizeMB?: number
  className?: string
}

const MediaUploader: React.FC<MediaUploaderProps> = ({
  eventId,
  venueId,
  type,
  onUploadComplete,
  maxFiles = 10,
  maxSizeMB = 10,
  className = ''
}) => {
  const [files, setFiles] = useState<File[]>([])
  const [uploading, setUploading] = useState(false)
  const [uploadProgress, setUploadProgress] = useState<Record<string, number>>({})
  const [errors, setErrors] = useState<string[]>([])
  const fileInputRef = useRef<HTMLInputElement>(null)

  const acceptedTypes = type === 'image' 
    ? 'image/jpeg,image/png,image/webp,image/gif'
    : type === 'document'
    ? '.pdf,.doc,.docx,.xls,.xlsx'
    : 'image/jpeg,image/png,image/webp,image/gif,.pdf,.doc,.docx,.xls,.xlsx'

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(event.target.files || [])
    const validFiles: File[] = []
    const newErrors: string[] = []

    selectedFiles.forEach(file => {
      const validation = mediaService.validateFile(file, maxSizeMB)
      if (validation.valid) {
        // Check file type matches expected type
        if (type === 'image' && !file.type.startsWith('image/')) {
          newErrors.push(`${file.name}: Only images are allowed`)
        } else if (type === 'document' && file.type.startsWith('image/')) {
          newErrors.push(`${file.name}: Only documents are allowed`)
        } else {
          validFiles.push(file)
        }
      } else {
        newErrors.push(`${file.name}: ${validation.error}`)
      }
    })

    // Check total file count
    if (files.length + validFiles.length > maxFiles) {
      newErrors.push(`Maximum ${maxFiles} files allowed`)
      const allowedCount = maxFiles - files.length
      validFiles.splice(allowedCount)
    }

    setFiles(prev => [...prev, ...validFiles])
    setErrors(newErrors)
  }

  const removeFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index))
  }

  const clearAll = () => {
    setFiles([])
    setErrors([])
    setUploadProgress({})
  }

  const handleUpload = async () => {
    if (files.length === 0) return

    setUploading(true)
    setErrors([])

    try {
      // Separate images and documents
      const imageFiles = files.filter(f => f.type.startsWith('image/'))
      const documentFiles = files.filter(f => !f.type.startsWith('image/'))

      const uploadedFiles: MediaFile[] = []

      // Upload images
      if (imageFiles.length > 0) {
        const imageFileList = new DataTransfer()
        imageFiles.forEach(file => imageFileList.items.add(file))

        if (eventId) {
          const result = await mediaService.uploadEventImages(eventId, imageFileList.files)
          if (result.uploadedImages) {
            uploadedFiles.push(...result.uploadedImages)
          }
        } else if (venueId) {
          const result = await mediaService.uploadVenueImages(venueId, imageFileList.files)
          if (result.uploadedImages) {
            uploadedFiles.push(...result.uploadedImages)
          }
        }
      }

      // Upload documents
      if (documentFiles.length > 0 && eventId) {
        const documentFileList = new DataTransfer()
        documentFiles.forEach(file => documentFileList.items.add(file))

        const result = await mediaService.uploadEventDocuments(eventId, documentFileList.files)
        if (result.uploadedDocuments) {
          uploadedFiles.push(...result.uploadedDocuments)
        }
      }

      // Clear files after successful upload
      setFiles([])
      onUploadComplete?.(uploadedFiles)

    } catch (error: any) {
      setErrors([error.message])
    } finally {
      setUploading(false)
    }
  }

  const handleDrop = (event: React.DragEvent) => {
    event.preventDefault()
    const droppedFiles = Array.from(event.dataTransfer.files)
    const fileInput = fileInputRef.current
    if (fileInput) {
      const dataTransfer = new DataTransfer()
      droppedFiles.forEach(file => dataTransfer.items.add(file))
      fileInput.files = dataTransfer.files
      handleFileSelect({ target: fileInput } as React.ChangeEvent<HTMLInputElement>)
    }
  }

  const handleDragOver = (event: React.DragEvent) => {
    event.preventDefault()
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Upload Area */}
      <div
        className="border-2 border-dashed border-luxury-gold/30 rounded-2xl p-8 text-center hover:border-luxury-gold/50 transition-colors cursor-pointer"
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onClick={() => fileInputRef.current?.click()}
      >
        <Upload className="h-12 w-12 text-luxury-gold mx-auto mb-4" />
        <h3 className="text-lg font-medium text-luxury-gold mb-2">
          Upload {type === 'both' ? 'Files' : type === 'image' ? 'Images' : 'Documents'}
        </h3>
        <p className="text-luxury-platinum/80 mb-4">
          Drag and drop files here, or click to select
        </p>
        <p className="text-luxury-platinum/60 text-sm">
          Maximum {maxFiles} files, {maxSizeMB}MB each
        </p>
        
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept={acceptedTypes}
          onChange={handleFileSelect}
          className="hidden"
        />
      </div>

      {/* Error Messages */}
      <AnimatePresence>
        {errors.length > 0 && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="bg-red-500/10 border border-red-500/20 rounded-lg p-4"
          >
            <div className="flex items-start space-x-2">
              <AlertCircle className="h-5 w-5 text-red-400 mt-0.5" />
              <div>
                <h4 className="text-red-400 font-medium mb-1">Upload Errors</h4>
                <ul className="text-red-300 text-sm space-y-1">
                  {errors.map((error, index) => (
                    <li key={index}>• {error}</li>
                  ))}
                </ul>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* File List */}
      <AnimatePresence>
        {files.length > 0 && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="luxury-glass rounded-2xl p-6"
          >
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-lg font-medium text-luxury-gold">
                Selected Files ({files.length})
              </h4>
              <button
                onClick={clearAll}
                className="text-luxury-platinum/60 hover:text-red-400 transition-colors"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            <div className="space-y-3">
              {files.map((file, index) => (
                <motion.div
                  key={`${file.name}-${index}`}
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                  className="flex items-center space-x-3 p-3 bg-luxury-midnight-black/30 rounded-lg"
                >
                  <div className="flex-shrink-0">
                    {file.type.startsWith('image/') ? (
                      <Image className="h-6 w-6 text-luxury-gold" />
                    ) : (
                      <File className="h-6 w-6 text-luxury-gold" />
                    )}
                  </div>
                  
                  <div className="flex-1 min-w-0">
                    <p className="text-luxury-platinum font-medium truncate">
                      {file.name}
                    </p>
                    <p className="text-luxury-platinum/60 text-sm">
                      {mediaService.formatFileSize(file.size)}
                    </p>
                  </div>

                  <button
                    onClick={() => removeFile(index)}
                    className="flex-shrink-0 text-luxury-platinum/60 hover:text-red-400 transition-colors"
                  >
                    <X className="h-4 w-4" />
                  </button>
                </motion.div>
              ))}
            </div>

            {/* Upload Actions */}
            <div className="flex items-center justify-between mt-6 pt-4 border-t border-luxury-gold/20">
              <div className="text-luxury-platinum/80 text-sm">
                Total size: {mediaService.formatFileSize(files.reduce((acc, file) => acc + file.size, 0))}
              </div>
              
              <div className="flex space-x-3">
                <button
                  onClick={clearAll}
                  disabled={uploading}
                  className="px-4 py-2 border border-luxury-gold/30 text-luxury-platinum rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50"
                >
                  Clear All
                </button>
                <button
                  onClick={handleUpload}
                  disabled={uploading || files.length === 0}
                  className="luxury-button px-6 py-2 disabled:opacity-50"
                >
                  {uploading ? (
                    <div className="flex items-center space-x-2">
                      <div className="w-4 h-4 border-2 border-luxury-midnight-black/30 border-t-luxury-midnight-black rounded-full animate-spin"></div>
                      <span>Uploading...</span>
                    </div>
                  ) : (
                    `Upload ${files.length} File${files.length !== 1 ? 's' : ''}`
                  )}
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

export default MediaUploader