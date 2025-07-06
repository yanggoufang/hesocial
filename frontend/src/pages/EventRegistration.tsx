import React, { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { eventService } from '../services/eventService'
import registrationService, { RegistrationRequest } from '../services/registrationService'
import { 
  Calendar, 
  MapPin, 
  Users, 
  Clock, 
  Shield, 
  Star, 
  Crown, 
  CreditCard,
  Check,
  X,
  AlertTriangle,
  ArrowLeft,
  User,
  Mail,
  Phone,
  Building
} from 'lucide-react'

interface Event {
  id: string
  name: string
  description: string
  dateTime: string
  registrationDeadline: string
  venueId: string
  categoryId: string
  organizerId: string
  pricing: any
  exclusivityLevel: 'VIP' | 'VVIP' | 'Invitation Only'
  dressCode: number
  capacity: number
  currentAttendees: number
  amenities: string[]
  privacyGuarantees: string[]
  images: string[]
  videoUrl?: string
  requirements: any[]
  isActive: boolean
  createdAt: string
  updatedAt: string
  venueName: string
  venueAddress: string
  categoryName: string
}

const EventRegistration: React.FC = () => {
  const { eventId } = useParams<{ eventId: string }>()
  const navigate = useNavigate()
  const { user, isAuthenticated } = useAuth()
  
  const [event, setEvent] = useState<Event | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [registering, setRegistering] = useState(false)
  const [registrationData, setRegistrationData] = useState<RegistrationRequest>({
    specialRequests: ''
  })
  const [eligibilityChecked, setEligibilityChecked] = useState(false)
  const [canRegister, setCanRegister] = useState(false)
  const [eligibilityError, setEligibilityError] = useState<string | null>(null)

  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login')
      return
    }
    
    if (eventId) {
      fetchEventDetails()
    }
  }, [eventId, isAuthenticated, navigate])

  const fetchEventDetails = async () => {
    if (!eventId) return
    
    setLoading(true)
    setError(null)
    
    try {
      const response = await eventService.getEventById(eventId)
      
      if (response.success && response.data) {
        setEvent(response.data)
        await checkEligibility()
      } else {
        setError(response.error || 'Failed to fetch event details')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const checkEligibility = async () => {
    if (!eventId) return
    
    try {
      const response = await registrationService.checkRegistrationEligibility(eventId)
      
      if (response.success) {
        setCanRegister(true)
        setEligibilityError(null)
      } else {
        setCanRegister(false)
        setEligibilityError(response.error || 'Cannot register for this event')
      }
    } catch (err) {
      setCanRegister(false)
      setEligibilityError('Failed to check eligibility')
    } finally {
      setEligibilityChecked(true)
    }
  }

  const handleRegistration = async () => {
    if (!eventId || !canRegister) return
    
    setRegistering(true)
    setError(null)
    
    try {
      const response = await registrationService.registerForEvent(eventId, registrationData)
      
      if (response.success) {
        // Navigate to registration confirmation or user registrations page
        navigate('/profile/registrations', { 
          state: { message: 'Registration submitted successfully!' }
        })
      } else {
        setError(response.error || 'Failed to register for event')
      }
    } catch (err) {
      setError('Network error occurred during registration')
    } finally {
      setRegistering(false)
    }
  }

  const getMembershipTierBadge = (tier: string) => {
    const styles = {
      'Platinum': 'bg-gray-100 text-gray-800 border-gray-300',
      'Diamond': 'bg-blue-100 text-blue-800 border-blue-300',
      'Black Card': 'bg-black text-white border-black'
    }
    
    const icons = {
      'Platinum': <Star className="w-3 h-3" />,
      'Diamond': <Crown className="w-3 h-3" />,
      'Black Card': <Shield className="w-3 h-3" />
    }

    return (
      <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border ${styles[tier as keyof typeof styles]}`}>
        {icons[tier as keyof typeof icons]}
        {tier}
      </span>
    )
  }

  const getExclusivityBadge = (level: string) => {
    const styles = {
      'VIP': 'bg-purple-100 text-purple-800 border-purple-300',
      'VVIP': 'bg-gold-100 text-gold-800 border-gold-300',
      'Invitation Only': 'bg-red-100 text-red-800 border-red-300'
    }

    return (
      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium border ${styles[level as keyof typeof styles] || 'bg-gray-100 text-gray-800 border-gray-300'}`}>
        {level}
      </span>
    )
  }

  const getDressCodeText = (code: number) => {
    const codes = {
      1: 'Casual',
      2: 'Business Casual',
      3: 'Business Formal',
      4: 'Cocktail Attire',
      5: 'Black Tie'
    }
    return codes[code as keyof typeof codes] || 'Not specified'
  }

  const formatPrice = (pricing: any) => {
    if (!pricing) return 'Price on request'
    
    const { vip, vvip, general, currency = 'TWD' } = pricing
    
    if (general) {
      return new Intl.NumberFormat('zh-TW', {
        style: 'currency',
        currency: currency
      }).format(general)
    }
    
    if (vip && vvip) {
      return `${new Intl.NumberFormat('zh-TW', { style: 'currency', currency }).format(vip)} - ${new Intl.NumberFormat('zh-TW', { style: 'currency', currency }).format(vvip)}`
    }
    
    if (vip) {
      return new Intl.NumberFormat('zh-TW', {
        style: 'currency',
        currency: currency
      }).format(vip)
    }
    
    return 'Price on request'
  }

  const formatDateTime = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('zh-TW', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      weekday: 'long'
    })
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading event details...</p>
        </div>
      </div>
    )
  }

  if (!event) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <AlertTriangle className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h1 className="text-2xl font-bold text-gray-900 mb-2">Event Not Found</h1>
          <p className="text-gray-600 mb-4">The event you're looking for doesn't exist or is no longer available.</p>
          <button
            onClick={() => navigate('/events')}
            className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Events
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-6">
          <button
            onClick={() => navigate('/events')}
            className="inline-flex items-center gap-2 text-gray-600 hover:text-gray-900 mb-4"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to Events
          </button>
          
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Event Registration</h1>
          <p className="text-gray-600">Complete your registration for this exclusive event</p>
        </div>

        {/* Error Message */}
        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
            <div className="flex items-center">
              <X className="w-5 h-5 text-red-600 mr-2" />
              <p className="text-red-800">{error}</p>
            </div>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Event Details */}
          <div className="lg:col-span-2">
            <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
              {/* Event Images */}
              {event.images && event.images.length > 0 && (
                <div className="aspect-video bg-gray-200">
                  <img
                    src={event.images[0]}
                    alt={event.name}
                    className="w-full h-full object-cover"
                  />
                </div>
              )}
              
              <div className="p-6">
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">{event.name}</h2>
                    <div className="flex items-center gap-3 mb-2">
                      {getExclusivityBadge(event.exclusivityLevel)}
                      <span className="text-gray-600">{event.categoryName}</span>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-2xl font-bold text-purple-600">{formatPrice(event.pricing)}</p>
                    <p className="text-sm text-gray-500">Per person</p>
                  </div>
                </div>

                <p className="text-gray-700 mb-6">{event.description}</p>

                {/* Event Info Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                  <div className="flex items-center gap-3">
                    <Calendar className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="font-medium text-gray-900">Date & Time</p>
                      <p className="text-gray-600">{formatDateTime(event.dateTime)}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3">
                    <MapPin className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="font-medium text-gray-900">Venue</p>
                      <p className="text-gray-600">{event.venueName}</p>
                      <p className="text-sm text-gray-500">{event.venueAddress}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3">
                    <Users className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="font-medium text-gray-900">Capacity</p>
                      <p className="text-gray-600">{event.currentAttendees} / {event.capacity} registered</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-3">
                    <Clock className="w-5 h-5 text-gray-400" />
                    <div>
                      <p className="font-medium text-gray-900">Dress Code</p>
                      <p className="text-gray-600">{getDressCodeText(event.dressCode)}</p>
                    </div>
                  </div>
                </div>

                {/* Registration Deadline */}
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="w-5 h-5 text-yellow-600" />
                    <p className="font-medium text-yellow-800">Registration Deadline</p>
                  </div>
                  <p className="text-yellow-700 mt-1">
                    {formatDateTime(event.registrationDeadline)}
                  </p>
                </div>

                {/* Requirements */}
                {event.requirements && event.requirements.length > 0 && (
                  <div className="mb-6">
                    <h3 className="text-lg font-medium text-gray-900 mb-3">Requirements</h3>
                    <div className="space-y-2">
                      {event.requirements.map((req, index) => (
                        <div key={index} className="flex items-center gap-2">
                          <Shield className="w-4 h-4 text-gray-400" />
                          <span className="text-gray-700">{req.description}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Amenities */}
                {event.amenities && event.amenities.length > 0 && (
                  <div className="mb-6">
                    <h3 className="text-lg font-medium text-gray-900 mb-3">Amenities</h3>
                    <div className="flex flex-wrap gap-2">
                      {event.amenities.map((amenity, index) => (
                        <span
                          key={index}
                          className="px-3 py-1 bg-gray-100 text-gray-700 rounded-full text-sm"
                        >
                          {amenity}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {/* Privacy Guarantees */}
                {event.privacyGuarantees && event.privacyGuarantees.length > 0 && (
                  <div>
                    <h3 className="text-lg font-medium text-gray-900 mb-3">Privacy & Security</h3>
                    <div className="space-y-2">
                      {event.privacyGuarantees.map((guarantee, index) => (
                        <div key={index} className="flex items-center gap-2">
                          <Check className="w-4 h-4 text-green-500" />
                          <span className="text-gray-700">{guarantee}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Registration Form */}
          <div className="lg:col-span-1">
            <div className="bg-white rounded-lg shadow-sm border p-6 sticky top-8">
              <h3 className="text-xl font-bold text-gray-900 mb-4">Registration</h3>

              {/* User Info */}
              {user && (
                <div className="mb-6 p-4 bg-gray-50 rounded-lg">
                  <h4 className="font-medium text-gray-900 mb-3">Your Information</h4>
                  <div className="space-y-2 text-sm">
                    <div className="flex items-center gap-2">
                      <User className="w-4 h-4 text-gray-400" />
                      <span>{user.firstName} {user.lastName}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Mail className="w-4 h-4 text-gray-400" />
                      <span>{user.email}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Building className="w-4 h-4 text-gray-400" />
                      <span>{user.profession}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Crown className="w-4 h-4 text-gray-400" />
                      {getMembershipTierBadge(user.membershipTier)}
                    </div>
                  </div>
                </div>
              )}

              {/* Eligibility Check */}
              {eligibilityChecked && !canRegister && (
                <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
                  <div className="flex items-center gap-2 mb-2">
                    <X className="w-5 h-5 text-red-600" />
                    <p className="font-medium text-red-800">Cannot Register</p>
                  </div>
                  <p className="text-red-700 text-sm">{eligibilityError}</p>
                </div>
              )}

              {/* Registration Form */}
              {canRegister && (
                <>
                  <div className="mb-4">
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Special Requests (Optional)
                    </label>
                    <textarea
                      value={registrationData.specialRequests}
                      onChange={(e) => setRegistrationData(prev => ({ ...prev, specialRequests: e.target.value }))}
                      rows={4}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      placeholder="Any dietary restrictions, accessibility needs, or special requests..."
                    />
                  </div>

                  <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
                    <h4 className="font-medium text-blue-900 mb-2">Registration Process</h4>
                    <ol className="list-decimal list-inside text-sm text-blue-800 space-y-1">
                      <li>Submit registration request</li>
                      <li>Admin review and approval</li>
                      <li>Payment processing</li>
                      <li>Confirmation and event details</li>
                    </ol>
                  </div>

                  <button
                    onClick={handleRegistration}
                    disabled={registering || !canRegister}
                    className="w-full px-4 py-3 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium flex items-center justify-center gap-2"
                  >
                    {registering ? (
                      <>
                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                        Submitting...
                      </>
                    ) : (
                      <>
                        <CreditCard className="w-4 h-4" />
                        Submit Registration
                      </>
                    )}
                  </button>

                  <p className="text-xs text-gray-500 mt-3 text-center">
                    Registration is subject to approval. You will be notified via email once your application is reviewed.
                  </p>
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default EventRegistration