import React, { useState, useEffect } from 'react'
import { Plus, Search, MapPin, Users, Edit, Trash2, ArrowLeft, Building, Phone, Mail, Image } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import eventService, { Venue, VenueFilters } from '../services/eventService'
import MediaGallery from '../components/MediaGallery'

const VenueManagement: React.FC = () => {
  const { user } = useAuth()
  const navigate = useNavigate()
  const [venues, setVenues] = useState<Venue[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [filters, setFilters] = useState<VenueFilters>({
    page: 1,
    limit: 20
  })
  const [pagination, setPagination] = useState({
    page: 1,
    limit: 20,
    total: 0,
    totalPages: 0
  })
  const [selectedVenue, setSelectedVenue] = useState<Venue | null>(null)
  const [showDeleteModal, setShowDeleteModal] = useState(false)
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  useEffect(() => {
    if (user && ['admin', 'super_admin'].includes(user.role)) {
      loadVenues()
    }
  }, [user, filters])

  const loadVenues = async () => {
    try {
      setLoading(true)
      setError(null)

      const response = await eventService.getVenues(filters)

      if (response.success) {
        setVenues(response.data)
        if (response.pagination) {
          setPagination(response.pagination)
        }
      }
    } catch (err: any) {
      setError(err.message || 'Failed to load venues')
    } finally {
      setLoading(false)
    }
  }

  const handleSearch = (searchTerm: string) => {
    setFilters(prev => ({
      ...prev,
      search: searchTerm,
      page: 1
    }))
  }

  const handleFilterChange = (key: keyof VenueFilters, value: string) => {
    setFilters(prev => ({
      ...prev,
      [key]: value || undefined,
      page: 1
    }))
  }

  const handlePageChange = (newPage: number) => {
    setFilters(prev => ({
      ...prev,
      page: newPage
    }))
  }

  const handleDeleteVenue = async () => {
    if (!selectedVenue) return

    try {
      setActionLoading(selectedVenue.id)
      await eventService.deleteVenue(selectedVenue.id)
      setShowDeleteModal(false)
      setSelectedVenue(null)
      await loadVenues()
    } catch (err: any) {
      setError(err.message || 'Failed to delete venue')
    } finally {
      setActionLoading(null)
    }
  }

  const getVenueTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      restaurant: 'bg-orange-100 text-orange-800',
      yacht: 'bg-blue-100 text-blue-800',
      gallery: 'bg-purple-100 text-purple-800',
      private_residence: 'bg-green-100 text-green-800',
      hotel: 'bg-indigo-100 text-indigo-800',
      outdoor: 'bg-emerald-100 text-emerald-800'
    }
    return colors[type] || 'bg-gray-100 text-gray-800'
  }

  const getPriceTierColor = (tier: string) => {
    const colors: Record<string, string> = {
      premium: 'bg-yellow-100 text-yellow-800',
      luxury: 'bg-amber-100 text-amber-800',
      ultra_luxury: 'bg-yellow-100 text-yellow-900 border border-yellow-300'
    }
    return colors[tier] || 'bg-gray-100 text-gray-800'
  }

  if (!user || !['admin', 'super_admin'].includes(user.role)) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Access Denied</h1>
          <p className="text-gray-600">You need admin privileges to access this page.</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="sm:flex sm:items-center">
          <div className="sm:flex-auto">
            <div className="flex items-center space-x-4 mb-2">
              <button
                onClick={() => navigate('/events/manage')}
                className="inline-flex items-center text-sm text-gray-500 hover:text-gray-700"
              >
                <ArrowLeft className="h-4 w-4 mr-1" />
                Back to Events
              </button>
            </div>
            <h1 className="text-2xl font-semibold text-gray-900">Venue Management</h1>
            <p className="mt-2 text-sm text-gray-700">
              Manage luxury venues for HeSocial events and experiences.
            </p>
          </div>
          <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
            <button
              type="button"
              onClick={() => {/* TODO: Open venue creation modal */}}
              className="inline-flex items-center justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 sm:w-auto"
            >
              <Plus className="h-4 w-4 mr-2" />
              Add Venue
            </button>
          </div>
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-red-50 p-4">
            <div className="flex">
              <div className="ml-3">
                <h3 className="text-sm font-medium text-red-800">Error</h3>
                <div className="mt-2 text-sm text-red-700">{error}</div>
              </div>
            </div>
          </div>
        )}

        {/* Filters */}
        <div className="mt-6 bg-white shadow rounded-lg p-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label htmlFor="search" className="block text-sm font-medium text-gray-700">
                Search Venues
              </label>
              <div className="mt-1 relative">
                <input
                  type="text"
                  id="search"
                  placeholder="Search by name, address, or city..."
                  value={filters.search || ''}
                  onChange={(e) => handleSearch(e.target.value)}
                  className="block w-full pr-10 border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
                <Search className="absolute right-3 top-2.5 h-4 w-4 text-gray-400" />
              </div>
            </div>

            <div>
              <label htmlFor="venueType" className="block text-sm font-medium text-gray-700">
                Venue Type
              </label>
              <select
                id="venueType"
                value={filters.venueType || ''}
                onChange={(e) => handleFilterChange('venueType', e.target.value)}
                className="mt-1 block w-full border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              >
                <option value="">All Types</option>
                <option value="restaurant">Restaurant</option>
                <option value="yacht">Yacht</option>
                <option value="gallery">Gallery</option>
                <option value="private_residence">Private Residence</option>
                <option value="hotel">Hotel</option>
                <option value="outdoor">Outdoor</option>
              </select>
            </div>

            <div>
              <label htmlFor="city" className="block text-sm font-medium text-gray-700">
                City
              </label>
              <input
                type="text"
                id="city"
                placeholder="Filter by city..."
                value={filters.city || ''}
                onChange={(e) => handleFilterChange('city', e.target.value)}
                className="mt-1 block w-full border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              />
            </div>

            <div>
              <label htmlFor="isActive" className="block text-sm font-medium text-gray-700">
                Status
              </label>
              <select
                id="isActive"
                value={filters.isActive || ''}
                onChange={(e) => handleFilterChange('isActive', e.target.value)}
                className="mt-1 block w-full border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              >
                <option value="">All Statuses</option>
                <option value="true">Active</option>
                <option value="false">Inactive</option>
              </select>
            </div>
          </div>
        </div>

        {/* Venues List */}
        <div className="mt-6 bg-white shadow overflow-hidden sm:rounded-md">
          {loading ? (
            <div className="p-6 text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
              <p className="mt-2 text-sm text-gray-500">Loading venues...</p>
            </div>
          ) : venues.length === 0 ? (
            <div className="p-6 text-center">
              <Building className="h-12 w-12 text-gray-400 mx-auto" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">No venues found</h3>
              <p className="mt-1 text-sm text-gray-500">
                Get started by adding your first luxury venue.
              </p>
              <div className="mt-6">
                <button
                  type="button"
                  onClick={() => {/* TODO: Open venue creation modal */}}
                  className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Add Venue
                </button>
              </div>
            </div>
          ) : (
            <ul className="divide-y divide-gray-200">
              {venues.map((venue) => (
                <li key={venue.id}>
                  <div className="px-4 py-4 sm:px-6">
                    <div className="flex items-center justify-between">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center space-x-3">
                          <h3 className="text-lg font-medium text-gray-900 truncate">
                            {venue.name}
                          </h3>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getVenueTypeColor(venue.venue_type)}`}>
                            {venue.venue_type.replace('_', ' ')}
                          </span>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getPriceTierColor(venue.price_tier)}`}>
                            {venue.price_tier.replace('_', ' ')}
                          </span>
                          {!venue.is_active && (
                            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                              Inactive
                            </span>
                          )}
                        </div>
                        
                        <div className="mt-2 flex items-center text-sm text-gray-500 space-x-6">
                          <div className="flex items-center">
                            <MapPin className="h-4 w-4 mr-1" />
                            {venue.address}, {venue.city}
                          </div>
                          <div className="flex items-center">
                            <Users className="h-4 w-4 mr-1" />
                            {venue.capacity_min}-{venue.capacity_max} capacity
                          </div>
                          {venue.contact_phone && (
                            <div className="flex items-center">
                              <Phone className="h-4 w-4 mr-1" />
                              {venue.contact_phone}
                            </div>
                          )}
                          {venue.contact_email && (
                            <div className="flex items-center">
                              <Mail className="h-4 w-4 mr-1" />
                              {venue.contact_email}
                            </div>
                          )}
                        </div>

                        {venue.description && (
                          <p className="mt-2 text-sm text-gray-600 line-clamp-2">{venue.description}</p>
                        )}

                        {venue.amenities && venue.amenities.length > 0 && (
                          <div className="mt-2 flex flex-wrap gap-1">
                            {venue.amenities.slice(0, 5).map((amenity, index) => (
                              <span
                                key={index}
                                className="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium bg-gray-100 text-gray-800"
                              >
                                {amenity}
                              </span>
                            ))}
                            {venue.amenities.length > 5 && (
                              <span className="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium bg-gray-100 text-gray-600">
                                +{venue.amenities.length - 5} more
                              </span>
                            )}
                          </div>
                        )}

                        {/* Compact Venue Media Preview */}
                        <div className="mt-3">
                          <div className="flex items-center text-xs text-gray-400 mb-2">
                            <Image className="h-3 w-3 mr-1" />
                            Venue Images
                          </div>
                          <div className="max-w-2xl">
                            <MediaGallery
                              venueId={venue.id}
                              type="image"
                              editable={false}
                              className="max-h-20 overflow-hidden"
                            />
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center space-x-2">
                        <button
                          onClick={() => {/* TODO: View venue details */}}
                          className="inline-flex items-center px-3 py-1 border border-gray-300 text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        >
                          <Building className="h-3 w-3 mr-1" />
                          View
                        </button>

                        <button
                          onClick={() => {/* TODO: Edit venue */}}
                          className="inline-flex items-center px-3 py-1 border border-gray-300 text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        >
                          <Edit className="h-3 w-3 mr-1" />
                          Edit
                        </button>

                        {user?.role === 'super_admin' && (
                          <button
                            onClick={() => {
                              setSelectedVenue(venue)
                              setShowDeleteModal(true)
                            }}
                            className="inline-flex items-center px-3 py-1 border border-gray-300 text-xs font-medium rounded-md text-red-700 bg-white hover:bg-red-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
                          >
                            <Trash2 className="h-3 w-3 mr-1" />
                            Delete
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Pagination */}
        {pagination.totalPages > 1 && (
          <div className="mt-6 flex items-center justify-between">
            <div className="flex-1 flex justify-between sm:hidden">
              <button
                onClick={() => handlePageChange(pagination.page - 1)}
                disabled={pagination.page <= 1}
                className="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              <button
                onClick={() => handlePageChange(pagination.page + 1)}
                disabled={pagination.page >= pagination.totalPages}
                className="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
            <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
              <div>
                <p className="text-sm text-gray-700">
                  Showing{' '}
                  <span className="font-medium">{((pagination.page - 1) * pagination.limit) + 1}</span>
                  {' '}to{' '}
                  <span className="font-medium">
                    {Math.min(pagination.page * pagination.limit, pagination.total)}
                  </span>
                  {' '}of{' '}
                  <span className="font-medium">{pagination.total}</span>
                  {' '}results
                </p>
              </div>
              <div>
                <nav className="relative z-0 inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
                  <button
                    onClick={() => handlePageChange(pagination.page - 1)}
                    disabled={pagination.page <= 1}
                    className="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Previous
                  </button>
                  {Array.from({ length: Math.min(5, pagination.totalPages) }, (_, i) => {
                    const pageNum = Math.max(1, Math.min(pagination.totalPages - 4, pagination.page - 2)) + i
                    return (
                      <button
                        key={pageNum}
                        onClick={() => handlePageChange(pageNum)}
                        className={`relative inline-flex items-center px-4 py-2 border text-sm font-medium ${
                          pageNum === pagination.page
                            ? 'z-10 bg-blue-50 border-blue-500 text-blue-600'
                            : 'bg-white border-gray-300 text-gray-500 hover:bg-gray-50'
                        }`}
                      >
                        {pageNum}
                      </button>
                    )
                  })}
                  <button
                    onClick={() => handlePageChange(pagination.page + 1)}
                    disabled={pagination.page >= pagination.totalPages}
                    className="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Next
                  </button>
                </nav>
              </div>
            </div>
          </div>
        )}

        {/* Delete Confirmation Modal */}
        {showDeleteModal && selectedVenue && (
          <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div className="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
              <div className="mt-3 text-center">
                <div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100">
                  <Trash2 className="h-6 w-6 text-red-600" />
                </div>
                <h3 className="text-lg font-medium text-gray-900 mt-4">Delete Venue</h3>
                <div className="mt-2 px-7 py-3">
                  <p className="text-sm text-gray-500">
                    Are you sure you want to delete "{selectedVenue.name}"? This action cannot be undone and will affect any events using this venue.
                  </p>
                </div>
                <div className="items-center px-4 py-3">
                  <button
                    onClick={handleDeleteVenue}
                    disabled={actionLoading === selectedVenue.id}
                    className="px-4 py-2 bg-red-500 text-white text-base font-medium rounded-md w-full shadow-sm hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-300 disabled:opacity-50"
                  >
                    {actionLoading === selectedVenue.id ? 'Deleting...' : 'Delete Venue'}
                  </button>
                  <button
                    onClick={() => {
                      setShowDeleteModal(false)
                      setSelectedVenue(null)
                    }}
                    className="mt-3 px-4 py-2 bg-white text-gray-500 text-base font-medium rounded-md w-full shadow-sm border border-gray-300 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-300"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default VenueManagement