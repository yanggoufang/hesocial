import React, { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { motion } from 'framer-motion'
import { ArrowLeft, Image, FileText, Upload, Eye, Trash2, Download } from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { usePermissions } from '../hooks/useRoleAccess'
import eventService, { Event } from '../services/eventService'
import MediaUploader from '../components/MediaUploader'
import MediaGallery from '../components/MediaGallery'

const EventMediaManagement: React.FC = () => {
  const { eventId } = useParams<{ eventId: string }>()
  const navigate = useNavigate()
  const { user } = useAuth()
  const { can } = usePermissions()
  const [event, setEvent] = useState<Event | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'images' | 'documents' | 'all'>('all')
  const [mediaRefresh, setMediaRefresh] = useState(0)

  useEffect(() => {
    if (!can.viewAdmin) {
      navigate('/events')
      return
    }

    if (eventId) {
      loadEvent()
    }
  }, [eventId, can.viewAdmin, navigate])

  const loadEvent = async () => {
    if (!eventId) return

    try {
      setLoading(true)
      setError(null)
      
      const response = await eventService.getEvent(eventId)
      if (response.success) {
        setEvent(response.data)
      } else {
        setError('Failed to load event')
      }
    } catch (err) {
      setError('Error loading event')
    } finally {
      setLoading(false)
    }
  }

  const handleMediaUpload = () => {
    setMediaRefresh(prev => prev + 1)
  }

  const handleMediaChange = () => {
    setMediaRefresh(prev => prev + 1)
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-luxury-gold"></div>
          </div>
        </div>
      </div>
    )
  }

  if (error || !event) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="luxury-glass p-8 rounded-2xl text-center">
            <h2 className="text-2xl font-luxury font-bold text-luxury-gold mb-4">
              Event Not Found
            </h2>
            <p className="text-luxury-platinum/80 mb-6">{error || 'Event not found'}</p>
            <button
              onClick={() => navigate('/events/manage')}
              className="luxury-button-primary"
            >
              <ArrowLeft className="h-4 w-4 mr-2" />
              Back to Events
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-12">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-center justify-between mb-8"
        >
          <div className="flex items-center space-x-4">
            <button
              onClick={() => navigate('/events/manage')}
              className="luxury-button-secondary flex items-center space-x-2"
            >
              <ArrowLeft className="h-4 w-4" />
              <span>Back to Events</span>
            </button>
            <div>
              <h1 className="text-3xl font-luxury font-bold text-luxury-gold">
                Media Management
              </h1>
              <p className="text-luxury-platinum/80 text-lg">
                {event.title}
              </p>
            </div>
          </div>
        </motion.div>

        {/* Event Info */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="luxury-glass p-6 rounded-2xl mb-8"
        >
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <h3 className="text-sm font-medium text-luxury-platinum/60">Event Date</h3>
              <p className="text-luxury-platinum">
                {new Date(event.start_datetime).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                  hour: '2-digit',
                  minute: '2-digit'
                })}
              </p>
            </div>
            <div>
              <h3 className="text-sm font-medium text-luxury-platinum/60">Venue</h3>
              <p className="text-luxury-platinum">{event.venue_name || 'TBD'}</p>
            </div>
            <div>
              <h3 className="text-sm font-medium text-luxury-platinum/60">Status</h3>
              <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                event.status === 'published' ? 'bg-green-100 text-green-800' :
                event.status === 'draft' ? 'bg-gray-100 text-gray-800' :
                'bg-yellow-100 text-yellow-800'
              }`}>
                {event.status.replace('_', ' ')}
              </span>
            </div>
          </div>
        </motion.div>

        {/* Tab Navigation */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="flex space-x-1 mb-8"
        >
          {[
            { id: 'all', label: 'All Media', icon: Eye },
            { id: 'images', label: 'Images', icon: Image },
            { id: 'documents', label: 'Documents', icon: FileText }
          ].map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id as any)}
              className={`flex items-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors ${
                activeTab === id
                  ? 'bg-luxury-gold text-luxury-midnight-black'
                  : 'text-luxury-platinum hover:bg-luxury-gold/10'
              }`}
            >
              <Icon className="h-4 w-4" />
              <span>{label}</span>
            </button>
          ))}
        </motion.div>

        {/* Upload Section */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="luxury-glass p-6 rounded-2xl mb-8"
        >
          <h2 className="text-xl font-luxury font-semibold text-luxury-gold mb-6 flex items-center">
            <Upload className="h-5 w-5 mr-2" />
            Upload New Media
          </h2>
          
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Image Upload */}
            <div>
              <h3 className="text-lg font-medium text-luxury-platinum mb-4">Event Images</h3>
              <MediaUploader
                eventId={event.id}
                type="image"
                maxFiles={10}
                maxSizeMB={10}
                onUploadComplete={handleMediaUpload}
                className="border-dashed border-2 border-luxury-gold/30 rounded-lg p-6 bg-luxury-gold/5"
              />
              <p className="text-sm text-luxury-platinum/60 mt-2">
                Upload high-quality images for event galleries and marketing. 
                Supported formats: JPEG, PNG, WebP, GIF. Max 10MB per file.
              </p>
            </div>

            {/* Document Upload */}
            <div>
              <h3 className="text-lg font-medium text-luxury-platinum mb-4">Event Documents</h3>
              <MediaUploader
                eventId={event.id}
                type="document"
                maxFiles={5}
                maxSizeMB={10}
                onUploadComplete={handleMediaUpload}
                className="border-dashed border-2 border-luxury-gold/30 rounded-lg p-6 bg-luxury-gold/5"
              />
              <p className="text-sm text-luxury-platinum/60 mt-2">
                Upload event-related documents like schedules, menus, or information packets.
                Supported formats: PDF, DOC, XLS. Max 10MB per file.
              </p>
            </div>
          </div>
        </motion.div>

        {/* Media Gallery */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <h2 className="text-xl font-luxury font-semibold text-luxury-gold mb-6">
            Current Media
          </h2>
          
          <MediaGallery
            eventId={event.id}
            type={activeTab === 'all' ? 'all' : activeTab}
            editable={true}
            onMediaChange={handleMediaChange}
            key={`${mediaRefresh}-${activeTab}`}
            className="min-h-48"
          />
        </motion.div>
      </div>
    </div>
  )
}

export default EventMediaManagement