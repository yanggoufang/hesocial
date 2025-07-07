import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  Image, File, Download, Trash2, Eye, X, 
  ChevronLeft, ChevronRight, Calendar, User, AlertCircle
} from 'lucide-react'
import { mediaService, MediaFile } from '../services/mediaService'
import { useAuth } from '../hooks/useAuth'

interface MediaGalleryProps {
  eventId?: string
  venueId?: string
  type?: 'image' | 'document' | 'all'
  editable?: boolean
  onMediaChange?: () => void
  className?: string
}

const MediaGallery: React.FC<MediaGalleryProps> = ({
  eventId,
  venueId,
  type = 'all',
  editable = false,
  onMediaChange,
  className = ''
}) => {
  const { user } = useAuth()
  const [media, setMedia] = useState<MediaFile[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedMedia, setSelectedMedia] = useState<MediaFile | null>(null)
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const [error, setError] = useState<string | null>(null)
  const [filter, setFilter] = useState<'all' | 'image' | 'document'>('all')

  useEffect(() => {
    loadMedia()
  }, [eventId, venueId, type])

  const loadMedia = async () => {
    try {
      setLoading(true)
      setError(null)
      
      let mediaData: MediaFile[] = []
      
      if (eventId) {
        const filterType = type === 'all' ? undefined : type
        mediaData = await mediaService.getEventMedia(eventId, filterType)
      } else if (venueId) {
        mediaData = await mediaService.getVenueMedia(venueId)
        // Filter for images only for venues
        mediaData = mediaData.filter(m => m.type === 'image')
      }
      
      setMedia(mediaData)
    } catch (error: any) {
      setError(error.message)
    } finally {
      setLoading(false)
    }
  }

  const handleDelete = async (mediaId: string) => {
    if (!confirm('Are you sure you want to delete this file?')) return
    
    try {
      await mediaService.deleteMedia(mediaId)
      setMedia(prev => prev.filter(m => m.id !== mediaId))
      onMediaChange?.()
      
      // Close modal if deleted media was selected
      if (selectedMedia?.id === mediaId) {
        setSelectedMedia(null)
        setSelectedIndex(-1)
      }
    } catch (error: any) {
      alert('Failed to delete file: ' + error.message)
    }
  }

  const handleDownload = async (media: MediaFile) => {
    try {
      if (media.type === 'document') {
        const { downloadUrl } = await mediaService.downloadDocument(media.id)
        window.open(downloadUrl, '_blank')
      } else {
        // For images, direct download
        const link = document.createElement('a')
        link.href = media.filePath
        link.download = media.originalFilename
        link.click()
      }
    } catch (error: any) {
      alert('Failed to download file: ' + error.message)
    }
  }

  const openLightbox = (media: MediaFile, index: number) => {
    if (media.type === 'image') {
      setSelectedMedia(media)
      setSelectedIndex(index)
    }
  }

  const navigateLightbox = (direction: 'prev' | 'next') => {
    const imageMedia = filteredMedia.filter(m => m.type === 'image')
    let newIndex = selectedIndex
    
    if (direction === 'prev') {
      newIndex = selectedIndex > 0 ? selectedIndex - 1 : imageMedia.length - 1
    } else {
      newIndex = selectedIndex < imageMedia.length - 1 ? selectedIndex + 1 : 0
    }
    
    setSelectedIndex(newIndex)
    setSelectedMedia(imageMedia[newIndex])
  }

  const filteredMedia = media.filter(m => {
    if (filter === 'all') return true
    return m.type === filter
  })

  const canEdit = editable && user && ['admin', 'super_admin'].includes(user.role)

  if (loading) {
    return (
      <div className={`flex items-center justify-center py-12 ${className}`}>
        <div className="luxury-glass p-8 rounded-2xl text-center">
          <div className="w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-luxury-platinum">Loading media...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className={`luxury-glass p-6 rounded-2xl ${className}`}>
        <div className="text-center text-red-400">
          <AlertCircle className="h-8 w-8 mx-auto mb-2" />
          <p>Failed to load media: {error}</p>
          <button 
            onClick={loadMedia}
            className="mt-4 luxury-button-secondary"
          >
            Try Again
          </button>
        </div>
      </div>
    )
  }

  if (media.length === 0) {
    return (
      <div className={`luxury-glass p-8 rounded-2xl text-center ${className}`}>
        <div className="text-luxury-platinum/60">
          <Image className="h-12 w-12 mx-auto mb-4 opacity-50" />
          <p>No media files found</p>
        </div>
      </div>
    )
  }

  return (
    <div className={className}>
      {/* Filter Tabs */}
      {type === 'all' && (
        <div className="flex space-x-2 mb-6">
          {['all', 'image', 'document'].map(filterType => (
            <button
              key={filterType}
              onClick={() => setFilter(filterType as any)}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                filter === filterType
                  ? 'bg-luxury-gold text-luxury-midnight-black'
                  : 'text-luxury-platinum/70 hover:text-luxury-gold'
              }`}
            >
              {filterType === 'all' ? 'All' : filterType === 'image' ? 'Images' : 'Documents'}
              <span className="ml-2 text-sm">
                ({filterType === 'all' ? media.length : media.filter(m => m.type === filterType).length})
              </span>
            </button>
          ))}
        </div>
      )}

      {/* Media Grid */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        <AnimatePresence>
          {filteredMedia.map((mediaFile, index) => (
            <motion.div
              key={mediaFile.id}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              className="luxury-glass rounded-xl overflow-hidden group relative"
            >
              {/* Media Preview */}
              <div className="aspect-square relative">
                {mediaFile.type === 'image' ? (
                  <img
                    src={mediaFile.thumbnails?.medium || mediaFile.filePath}
                    alt={mediaFile.originalFilename}
                    className="w-full h-full object-cover cursor-pointer"
                    onClick={() => openLightbox(mediaFile, index)}
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center bg-luxury-midnight-black/30">
                    <div className="text-center">
                      <File className="h-12 w-12 text-luxury-gold mx-auto mb-2" />
                      <p className="text-luxury-gold text-2xl">
                        {mediaService.getFileTypeIcon(mediaFile.mimeType)}
                      </p>
                    </div>
                  </div>
                )}

                {/* Hover Actions */}
                <div className="absolute inset-0 bg-luxury-midnight-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center space-x-2">
                  {mediaFile.type === 'image' && (
                    <button
                      onClick={() => openLightbox(mediaFile, index)}
                      className="w-10 h-10 bg-luxury-gold/20 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-gold/30 transition-colors"
                    >
                      <Eye className="h-5 w-5" />
                    </button>
                  )}
                  
                  <button
                    onClick={() => handleDownload(mediaFile)}
                    className="w-10 h-10 bg-luxury-gold/20 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-gold/30 transition-colors"
                  >
                    <Download className="h-5 w-5" />
                  </button>

                  {canEdit && (
                    <button
                      onClick={() => handleDelete(mediaFile.id)}
                      className="w-10 h-10 bg-red-500/20 backdrop-blur-sm rounded-full flex items-center justify-center text-red-400 hover:bg-red-500/30 transition-colors"
                    >
                      <Trash2 className="h-5 w-5" />
                    </button>
                  )}
                </div>
              </div>

              {/* Media Info */}
              <div className="p-3">
                <h4 className="text-luxury-platinum font-medium text-sm truncate mb-1">
                  {mediaFile.originalFilename}
                </h4>
                <div className="flex items-center justify-between text-xs text-luxury-platinum/60">
                  <span>{mediaService.formatFileSize(mediaFile.fileSize)}</span>
                  <span>{new Date(mediaFile.createdAt).toLocaleDateString()}</span>
                </div>
              </div>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      {/* Lightbox Modal */}
      <AnimatePresence>
        {selectedMedia && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-luxury-midnight-black/90 backdrop-blur-sm z-50 flex items-center justify-center p-4"
            onClick={() => setSelectedMedia(null)}
          >
            <motion.div
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
              className="max-w-5xl max-h-full flex flex-col"
              onClick={e => e.stopPropagation()}
            >
              {/* Header */}
              <div className="flex items-center justify-between p-4 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-t-2xl">
                <h3 className="text-luxury-gold font-medium">
                  {selectedMedia.originalFilename}
                </h3>
                <button
                  onClick={() => setSelectedMedia(null)}
                  className="text-luxury-platinum/60 hover:text-luxury-gold transition-colors"
                >
                  <X className="h-6 w-6" />
                </button>
              </div>

              {/* Image */}
              <div className="relative">
                <img
                  src={selectedMedia.filePath}
                  alt={selectedMedia.originalFilename}
                  className="max-w-full max-h-[70vh] object-contain"
                />

                {/* Navigation */}
                {filteredMedia.filter(m => m.type === 'image').length > 1 && (
                  <>
                    <button
                      onClick={() => navigateLightbox('prev')}
                      className="absolute left-4 top-1/2 transform -translate-y-1/2 w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors"
                    >
                      <ChevronLeft className="h-6 w-6" />
                    </button>
                    <button
                      onClick={() => navigateLightbox('next')}
                      className="absolute right-4 top-1/2 transform -translate-y-1/2 w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors"
                    >
                      <ChevronRight className="h-6 w-6" />
                    </button>
                  </>
                )}
              </div>

              {/* Footer */}
              <div className="flex items-center justify-between p-4 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-b-2xl">
                <div className="flex items-center space-x-4 text-luxury-platinum/80 text-sm">
                  <span className="flex items-center space-x-1">
                    <File className="h-4 w-4" />
                    <span>{mediaService.formatFileSize(selectedMedia.fileSize)}</span>
                  </span>
                  <span className="flex items-center space-x-1">
                    <Calendar className="h-4 w-4" />
                    <span>{new Date(selectedMedia.createdAt).toLocaleDateString()}</span>
                  </span>
                </div>

                <div className="flex space-x-2">
                  <button
                    onClick={() => handleDownload(selectedMedia)}
                    className="luxury-button-secondary px-4 py-2"
                  >
                    <Download className="h-4 w-4 mr-2" />
                    Download
                  </button>
                  {canEdit && (
                    <button
                      onClick={() => handleDelete(selectedMedia.id)}
                      className="px-4 py-2 bg-red-500/20 text-red-400 rounded-lg hover:bg-red-500/30 transition-colors"
                    >
                      <Trash2 className="h-4 w-4 mr-2" />
                      Delete
                    </button>
                  )}
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

export default MediaGallery