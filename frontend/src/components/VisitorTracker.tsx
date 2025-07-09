import React, { useEffect, useState } from 'react'

interface VisitorTrackerProps {
  showVisitorId?: boolean
}

const VisitorTracker: React.FC<VisitorTrackerProps> = ({ showVisitorId = false }) => {
  const [visitorId, setVisitorId] = useState<string | null>(null)

  useEffect(() => {
    // Get visitor ID from cookie
    const getVisitorId = () => {
      const cookies = document.cookie.split(';')
      for (let cookie of cookies) {
        const [name, value] = cookie.trim().split('=')
        if (name === 'visitorId') {
          return decodeURIComponent(value)
        }
      }
      return null
    }

    const id = getVisitorId()
    setVisitorId(id)

    // Set visitor ID in headers for API calls
    if (id) {
      // Add to axios defaults if available
      if (window.axios) {
        window.axios.defaults.headers.common['X-Visitor-ID'] = id
      }
    }
  }, [])

  // Track custom events (for future Google Analytics integration)
  const trackEvent = (eventType: string, eventData?: any) => {
    if (!visitorId) return

    // Send to backend analytics API
    fetch('/api/analytics/events/track', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Visitor-ID': visitorId
      },
      body: JSON.stringify({
        visitor_id: visitorId,
        event_type: eventType,
        event_data: eventData
      })
    }).catch(error => {
      console.warn('Failed to track event:', error)
    })
  }

  // Expose tracking function globally for easy use
  useEffect(() => {
    if (visitorId) {
      (window as any).trackVisitorEvent = trackEvent
    }
  }, [visitorId])

  if (!showVisitorId || !visitorId) {
    return null
  }

  return (
    <div className="fixed bottom-4 right-4 bg-black/80 text-white px-3 py-2 rounded-lg text-xs font-mono z-50">
      <div className="text-gray-300">Visitor ID:</div>
      <div className="text-yellow-400">{visitorId}</div>
    </div>
  )
}

export default VisitorTracker
