import React, { useState, useEffect } from 'react'
import { X, Calendar, MapPin, DollarSign, Users, Clock, Info, Save, AlertCircle } from 'lucide-react'
import eventService, { Event, EventCategory, Venue, CreateEventData } from '../services/eventService'

interface EventFormProps {
  event?: Event | null
  isOpen: boolean
  onClose: () => void
  onSuccess: () => void
}

const EventForm: React.FC<EventFormProps> = ({ event, isOpen, onClose, onSuccess }) => {
  const [categories, setCategories] = useState<EventCategory[]>([])
  const [venues, setVenues] = useState<Venue[]>([])
  const [loading, setLoading] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const [formData, setFormData] = useState<CreateEventData>({
    title: '',
    description: '',
    detailedDescription: '',
    categoryId: '',
    venueId: '',
    startDatetime: '',
    endDatetime: '',
    timezone: 'Asia/Taipei',
    capacityMin: 1,
    capacityMax: 20,
    pricePlatinum: 0,
    priceDiamond: 0,
    priceBlackCard: 0,
    currency: 'TWD',
    requiredMembershipTiers: [],
    requiredVerification: true,
    ageRestriction: {},
    dressCode: '',
    language: 'Traditional Chinese',
    specialRequirements: '',
    inclusions: [],
    exclusions: [],
    registrationOpensAt: '',
    registrationClosesAt: '',
    cancellationDeadline: '',
    waitlistEnabled: true,
    autoApproval: false,
    metaTitle: '',
    metaDescription: '',
    featuredImage: '',
    internalNotes: '',
    costBreakdown: {},
    profitMargin: 0
  })

  useEffect(() => {
    if (isOpen) {
      loadFormData()
    }
  }, [isOpen])

  useEffect(() => {
    if (event) {
      setFormData({
        title: event.title,
        description: event.description,
        detailedDescription: event.detailed_description || '',
        categoryId: event.category_id,
        venueId: event.venue_id,
        startDatetime: event.start_datetime.slice(0, 16),
        endDatetime: event.end_datetime.slice(0, 16),
        timezone: event.timezone,
        capacityMin: event.capacity_min,
        capacityMax: event.capacity_max,
        pricePlatinum: event.price_platinum,
        priceDiamond: event.price_diamond,
        priceBlackCard: event.price_black_card,
        currency: event.currency,
        requiredMembershipTiers: event.required_membership_tiers,
        requiredVerification: event.required_verification,
        ageRestriction: event.age_restriction || {},
        dressCode: event.dress_code || '',
        language: event.language,
        specialRequirements: event.special_requirements || '',
        inclusions: event.inclusions,
        exclusions: event.exclusions,
        registrationOpensAt: event.registration_opens_at?.slice(0, 16) || '',
        registrationClosesAt: event.registration_closes_at?.slice(0, 16) || '',
        cancellationDeadline: event.cancellation_deadline?.slice(0, 16) || '',
        waitlistEnabled: event.waitlist_enabled,
        autoApproval: event.auto_approval,
        metaTitle: event.meta_title || '',
        metaDescription: event.meta_description || '',
        featuredImage: event.featured_image || '',
        internalNotes: event.internal_notes || '',
        costBreakdown: event.cost_breakdown || {},
        profitMargin: event.profit_margin || 0
      })
    } else {
      // Reset form for new event
      setFormData({
        title: '',
        description: '',
        detailedDescription: '',
        categoryId: '',
        venueId: '',
        startDatetime: '',
        endDatetime: '',
        timezone: 'Asia/Taipei',
        capacityMin: 1,
        capacityMax: 20,
        pricePlatinum: 0,
        priceDiamond: 0,
        priceBlackCard: 0,
        currency: 'TWD',
        requiredMembershipTiers: [],
        requiredVerification: true,
        ageRestriction: {},
        dressCode: '',
        language: 'Traditional Chinese',
        specialRequirements: '',
        inclusions: [],
        exclusions: [],
        registrationOpensAt: '',
        registrationClosesAt: '',
        cancellationDeadline: '',
        waitlistEnabled: true,
        autoApproval: false,
        metaTitle: '',
        metaDescription: '',
        featuredImage: '',
        internalNotes: '',
        costBreakdown: {},
        profitMargin: 0
      })
    }
  }, [event])

  const loadFormData = async () => {
    try {
      setLoading(true)
      const [categoriesResponse, venuesResponse] = await Promise.all([
        eventService.getCategories(),
        eventService.getVenues({ limit: 100 })
      ])

      if (categoriesResponse.success) {
        setCategories(categoriesResponse.data)
      }
      if (venuesResponse.success) {
        setVenues(venuesResponse.data)
      }
    } catch (err: any) {
      setError(err.message || 'Failed to load form data')
    } finally {
      setLoading(false)
    }
  }

  const handleInputChange = (field: keyof CreateEventData, value: any) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }))
  }

  const handleArrayChange = (field: 'inclusions' | 'exclusions', value: string) => {
    const items = value.split('\n').filter(item => item.trim())
    handleInputChange(field, items)
  }

  const handleMembershipTierChange = (tier: string, checked: boolean) => {
    const updatedTiers = checked
      ? [...formData.requiredMembershipTiers, tier]
      : formData.requiredMembershipTiers.filter(t => t !== tier)
    handleInputChange('requiredMembershipTiers', updatedTiers)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    try {
      setSubmitting(true)
      setError(null)

      // Basic validation
      if (!formData.title.trim()) {
        throw new Error('Event title is required')
      }
      if (!formData.description.trim()) {
        throw new Error('Event description is required')
      }
      if (!formData.categoryId) {
        throw new Error('Event category is required')
      }
      if (!formData.venueId) {
        throw new Error('Venue is required')
      }
      if (!formData.startDatetime) {
        throw new Error('Start date and time is required')
      }
      if (!formData.endDatetime) {
        throw new Error('End date and time is required')
      }
      if (new Date(formData.startDatetime) >= new Date(formData.endDatetime)) {
        throw new Error('End time must be after start time')
      }
      if (formData.capacityMax <= 0) {
        throw new Error('Maximum capacity must be greater than 0')
      }
      if (formData.capacityMin > formData.capacityMax) {
        throw new Error('Minimum capacity cannot exceed maximum capacity')
      }

      // Convert datetime strings to ISO format
      const submitData = {
        ...formData,
        startDatetime: new Date(formData.startDatetime).toISOString(),
        endDatetime: new Date(formData.endDatetime).toISOString(),
        registrationOpensAt: formData.registrationOpensAt ? new Date(formData.registrationOpensAt).toISOString() : undefined,
        registrationClosesAt: formData.registrationClosesAt ? new Date(formData.registrationClosesAt).toISOString() : undefined,
        cancellationDeadline: formData.cancellationDeadline ? new Date(formData.cancellationDeadline).toISOString() : undefined
      }

      if (event) {
        await eventService.updateEvent(event.id, submitData)
      } else {
        await eventService.createEvent(submitData)
      }

      onSuccess()
      onClose()
    } catch (err: any) {
      setError(err.message || 'Failed to save event')
    } finally {
      setSubmitting(false)
    }
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-4 mx-auto p-5 border w-full max-w-4xl shadow-lg rounded-md bg-white mb-4">
        <div className="flex items-center justify-between border-b pb-4 mb-6">
          <h3 className="text-2xl font-semibold text-gray-900">
            {event ? 'Edit Event' : 'Create New Event'}
          </h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X className="h-6 w-6" />
          </button>
        </div>

        {error && (
          <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-md">
            <div className="flex">
              <AlertCircle className="h-5 w-5 text-red-400" />
              <div className="ml-3">
                <h3 className="text-sm font-medium text-red-800">Error</h3>
                <p className="mt-1 text-sm text-red-700">{error}</p>
              </div>
            </div>
          </div>
        )}

        {loading ? (
          <div className="text-center py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
            <p className="mt-2 text-sm text-gray-500">Loading form data...</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Basic Information */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="lg:col-span-2">
                <h4 className="text-lg font-medium text-gray-900 flex items-center mb-4">
                  <Info className="h-5 w-5 mr-2" />
                  Basic Information
                </h4>
              </div>

              <div>
                <label htmlFor="title" className="block text-sm font-medium text-gray-700">
                  Event Title *
                </label>
                <input
                  type="text"
                  id="title"
                  value={formData.title}
                  onChange={(e) => handleInputChange('title', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Enter event title"
                  required
                />
              </div>

              <div>
                <label htmlFor="categoryId" className="block text-sm font-medium text-gray-700">
                  Category *
                </label>
                <select
                  id="categoryId"
                  value={formData.categoryId}
                  onChange={(e) => handleInputChange('categoryId', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  required
                >
                  <option value="">Select a category</option>
                  {categories.map((category) => (
                    <option key={category.id} value={category.id}>
                      {category.name}
                    </option>
                  ))}
                </select>
              </div>

              <div className="lg:col-span-2">
                <label htmlFor="description" className="block text-sm font-medium text-gray-700">
                  Description *
                </label>
                <textarea
                  id="description"
                  value={formData.description}
                  onChange={(e) => handleInputChange('description', e.target.value)}
                  rows={3}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Brief description of the event"
                  required
                />
              </div>

              <div className="lg:col-span-2">
                <label htmlFor="detailedDescription" className="block text-sm font-medium text-gray-700">
                  Detailed Description
                </label>
                <textarea
                  id="detailedDescription"
                  value={formData.detailedDescription}
                  onChange={(e) => handleInputChange('detailedDescription', e.target.value)}
                  rows={5}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Detailed event description, agenda, and special notes"
                />
              </div>
            </div>

            {/* Date, Time & Venue */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="lg:col-span-2">
                <h4 className="text-lg font-medium text-gray-900 flex items-center mb-4">
                  <Calendar className="h-5 w-5 mr-2" />
                  Schedule & Venue
                </h4>
              </div>

              <div>
                <label htmlFor="venueId" className="block text-sm font-medium text-gray-700">
                  Venue *
                </label>
                <select
                  id="venueId"
                  value={formData.venueId}
                  onChange={(e) => handleInputChange('venueId', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  required
                >
                  <option value="">Select a venue</option>
                  {venues.map((venue) => (
                    <option key={venue.id} value={venue.id}>
                      {venue.name} - {venue.city} (Max: {venue.capacity_max})
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label htmlFor="timezone" className="block text-sm font-medium text-gray-700">
                  Timezone
                </label>
                <select
                  id="timezone"
                  value={formData.timezone}
                  onChange={(e) => handleInputChange('timezone', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                >
                  <option value="Asia/Taipei">Asia/Taipei</option>
                  <option value="Asia/Hong_Kong">Asia/Hong Kong</option>
                  <option value="Asia/Singapore">Asia/Singapore</option>
                  <option value="UTC">UTC</option>
                </select>
              </div>

              <div>
                <label htmlFor="startDatetime" className="block text-sm font-medium text-gray-700">
                  Start Date & Time *
                </label>
                <input
                  type="datetime-local"
                  id="startDatetime"
                  value={formData.startDatetime}
                  onChange={(e) => handleInputChange('startDatetime', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  required
                />
              </div>

              <div>
                <label htmlFor="endDatetime" className="block text-sm font-medium text-gray-700">
                  End Date & Time *
                </label>
                <input
                  type="datetime-local"
                  id="endDatetime"
                  value={formData.endDatetime}
                  onChange={(e) => handleInputChange('endDatetime', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  required
                />
              </div>
            </div>

            {/* Capacity & Pricing */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="lg:col-span-2">
                <h4 className="text-lg font-medium text-gray-900 flex items-center mb-4">
                  <Users className="h-5 w-5 mr-2" />
                  Capacity & Pricing
                </h4>
              </div>

              <div>
                <label htmlFor="capacityMin" className="block text-sm font-medium text-gray-700">
                  Minimum Capacity
                </label>
                <input
                  type="number"
                  id="capacityMin"
                  value={formData.capacityMin}
                  onChange={(e) => handleInputChange('capacityMin', parseInt(e.target.value) || 0)}
                  min="1"
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div>
                <label htmlFor="capacityMax" className="block text-sm font-medium text-gray-700">
                  Maximum Capacity *
                </label>
                <input
                  type="number"
                  id="capacityMax"
                  value={formData.capacityMax}
                  onChange={(e) => handleInputChange('capacityMax', parseInt(e.target.value) || 0)}
                  min="1"
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  required
                />
              </div>

              <div>
                <label htmlFor="pricePlatinum" className="block text-sm font-medium text-gray-700">
                  Platinum Price (TWD)
                </label>
                <input
                  type="number"
                  id="pricePlatinum"
                  value={formData.pricePlatinum}
                  onChange={(e) => handleInputChange('pricePlatinum', parseFloat(e.target.value) || 0)}
                  min="0"
                  step="0.01"
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div>
                <label htmlFor="priceDiamond" className="block text-sm font-medium text-gray-700">
                  Diamond Price (TWD)
                </label>
                <input
                  type="number"
                  id="priceDiamond"
                  value={formData.priceDiamond}
                  onChange={(e) => handleInputChange('priceDiamond', parseFloat(e.target.value) || 0)}
                  min="0"
                  step="0.01"
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div>
                <label htmlFor="priceBlackCard" className="block text-sm font-medium text-gray-700">
                  Black Card Price (TWD)
                </label>
                <input
                  type="number"
                  id="priceBlackCard"
                  value={formData.priceBlackCard}
                  onChange={(e) => handleInputChange('priceBlackCard', parseFloat(e.target.value) || 0)}
                  min="0"
                  step="0.01"
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Required Membership Tiers
                </label>
                <div className="space-y-2">
                  {['Platinum', 'Diamond', 'Black Card'].map((tier) => (
                    <label key={tier} className="inline-flex items-center mr-4">
                      <input
                        type="checkbox"
                        checked={formData.requiredMembershipTiers.includes(tier)}
                        onChange={(e) => handleMembershipTierChange(tier, e.target.checked)}
                        className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-offset-0 focus:ring-blue-200 focus:ring-opacity-50"
                      />
                      <span className="ml-2 text-sm text-gray-700">{tier}</span>
                    </label>
                  ))}
                </div>
              </div>
            </div>

            {/* Event Details */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="lg:col-span-2">
                <h4 className="text-lg font-medium text-gray-900 flex items-center mb-4">
                  <Clock className="h-5 w-5 mr-2" />
                  Event Details
                </h4>
              </div>

              <div>
                <label htmlFor="dressCode" className="block text-sm font-medium text-gray-700">
                  Dress Code
                </label>
                <input
                  type="text"
                  id="dressCode"
                  value={formData.dressCode}
                  onChange={(e) => handleInputChange('dressCode', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="e.g., Cocktail, Business formal"
                />
              </div>

              <div>
                <label htmlFor="language" className="block text-sm font-medium text-gray-700">
                  Language
                </label>
                <select
                  id="language"
                  value={formData.language}
                  onChange={(e) => handleInputChange('language', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                >
                  <option value="Traditional Chinese">Traditional Chinese</option>
                  <option value="English">English</option>
                  <option value="Bilingual">Bilingual</option>
                </select>
              </div>

              <div className="lg:col-span-2">
                <label htmlFor="inclusions" className="block text-sm font-medium text-gray-700">
                  Inclusions (one per line)
                </label>
                <textarea
                  id="inclusions"
                  value={formData.inclusions.join('\n')}
                  onChange={(e) => handleArrayChange('inclusions', e.target.value)}
                  rows={3}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Welcome drink&#10;Multi-course dinner&#10;Premium wine pairing"
                />
              </div>

              <div className="lg:col-span-2">
                <label htmlFor="exclusions" className="block text-sm font-medium text-gray-700">
                  Exclusions (one per line)
                </label>
                <textarea
                  id="exclusions"
                  value={formData.exclusions.join('\n')}
                  onChange={(e) => handleArrayChange('exclusions', e.target.value)}
                  rows={3}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Transportation&#10;Additional beverages&#10;Gratuity"
                />
              </div>
            </div>

            {/* Registration & Settings */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="lg:col-span-2">
                <h4 className="text-lg font-medium text-gray-900 flex items-center mb-4">
                  <DollarSign className="h-5 w-5 mr-2" />
                  Registration & Settings
                </h4>
              </div>

              <div>
                <label htmlFor="registrationOpensAt" className="block text-sm font-medium text-gray-700">
                  Registration Opens At
                </label>
                <input
                  type="datetime-local"
                  id="registrationOpensAt"
                  value={formData.registrationOpensAt}
                  onChange={(e) => handleInputChange('registrationOpensAt', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div>
                <label htmlFor="registrationClosesAt" className="block text-sm font-medium text-gray-700">
                  Registration Closes At
                </label>
                <input
                  type="datetime-local"
                  id="registrationClosesAt"
                  value={formData.registrationClosesAt}
                  onChange={(e) => handleInputChange('registrationClosesAt', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div>
                <label htmlFor="cancellationDeadline" className="block text-sm font-medium text-gray-700">
                  Cancellation Deadline
                </label>
                <input
                  type="datetime-local"
                  id="cancellationDeadline"
                  value={formData.cancellationDeadline}
                  onChange={(e) => handleInputChange('cancellationDeadline', e.target.value)}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
              </div>

              <div className="space-y-4">
                <label className="inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={formData.waitlistEnabled}
                    onChange={(e) => handleInputChange('waitlistEnabled', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-offset-0 focus:ring-blue-200 focus:ring-opacity-50"
                  />
                  <span className="ml-2 text-sm text-gray-700">Enable Waitlist</span>
                </label>

                <label className="inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={formData.autoApproval}
                    onChange={(e) => handleInputChange('autoApproval', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-offset-0 focus:ring-blue-200 focus:ring-opacity-50"
                  />
                  <span className="ml-2 text-sm text-gray-700">Auto-approve Registrations</span>
                </label>

                <label className="inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={formData.requiredVerification}
                    onChange={(e) => handleInputChange('requiredVerification', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-offset-0 focus:ring-blue-200 focus:ring-opacity-50"
                  />
                  <span className="ml-2 text-sm text-gray-700">Require User Verification</span>
                </label>
              </div>

              <div className="lg:col-span-2">
                <label htmlFor="internalNotes" className="block text-sm font-medium text-gray-700">
                  Internal Notes (Admin Only)
                </label>
                <textarea
                  id="internalNotes"
                  value={formData.internalNotes}
                  onChange={(e) => handleInputChange('internalNotes', e.target.value)}
                  rows={3}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  placeholder="Internal notes for admin reference"
                />
              </div>
            </div>

            {/* Form Actions */}
            <div className="flex items-center justify-end space-x-4 pt-6 border-t">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={submitting}
                className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Save className="h-4 w-4 mr-2" />
                {submitting ? 'Saving...' : (event ? 'Update Event' : 'Create Event')}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  )
}

export default EventForm