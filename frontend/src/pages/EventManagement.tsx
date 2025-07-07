import React, { useState, useEffect } from 'react'
import { Plus, Search, Calendar, MapPin, Users, Eye, Edit, Trash2, CheckCircle, XCircle, ExternalLink, ArrowLeft, Image } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import eventService, { Event, EventFilters, EventCategory, Venue } from '../services/eventService'
import EventForm from '../components/EventForm'
import MediaGallery from '../components/MediaGallery'

const EventManagement: React.FC = () => {
  const { user } = useAuth()
  const navigate = useNavigate()
  const [events, setEvents] = useState<Event[]>([])
  const [categories, setCategories] = useState<EventCategory[]>([])
  const [venues, setVenues] = useState<Venue[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [filters, setFilters] = useState<EventFilters>({
    page: 1,
    limit: 20
  })
  const [pagination, setPagination] = useState({
    page: 1,
    limit: 20,
    total: 0,
    totalPages: 0
  })
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null)
  const [showEventModal, setShowEventModal] = useState(false)
  const [showDeleteModal, setShowDeleteModal] = useState(false)
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  useEffect(() => {
    if (user && ['admin', 'super_admin'].includes(user.role)) {
      loadData()
    }
  }, [user, filters])

  const loadData = async () => {
    try {
      setLoading(true)
      setError(null)

      const [eventsResponse, categoriesResponse, venuesResponse] = await Promise.all([
        eventService.getEvents(filters),
        eventService.getCategories(),
        eventService.getVenues({ limit: 100 })
      ])

      if (eventsResponse.success) {
        setEvents(eventsResponse.data)
        if (eventsResponse.pagination) {
          setPagination(eventsResponse.pagination)
        }
      }

      if (categoriesResponse.success) {
        setCategories(categoriesResponse.data)
      }

      if (venuesResponse.success) {
        setVenues(venuesResponse.data)
      }
    } catch (err: any) {
      setError(err.message || 'Failed to load events')
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

  const handleFilterChange = (key: keyof EventFilters, value: string) => {
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

  const handleApproveEvent = async (eventId: string, approved: boolean) => {
    try {
      setActionLoading(eventId)
      await eventService.approveEvent(eventId, approved)
      await loadData()
    } catch (err: any) {
      setError(err.message || 'Failed to update event approval')
    } finally {
      setActionLoading(null)
    }
  }

  const handlePublishEvent = async (eventId: string) => {
    try {
      setActionLoading(eventId)
      await eventService.publishEvent(eventId)
      await loadData()
    } catch (err: any) {
      setError(err.message || 'Failed to publish event')
    } finally {
      setActionLoading(null)
    }
  }

  const handleDeleteEvent = async () => {
    if (!selectedEvent) return

    try {
      setActionLoading(selectedEvent.id)
      await eventService.deleteEvent(selectedEvent.id)
      setShowDeleteModal(false)
      setSelectedEvent(null)
      await loadData()
    } catch (err: any) {
      setError(err.message || 'Failed to delete event')
    } finally {
      setActionLoading(null)
    }
  }

  const getStatusColor = (status: Event['status']) => {
    const colors = {
      draft: 'bg-gray-100 text-gray-800',
      pending_review: 'bg-yellow-100 text-yellow-800',
      approved: 'bg-blue-100 text-blue-800',
      published: 'bg-green-100 text-green-800',
      full: 'bg-purple-100 text-purple-800',
      completed: 'bg-gray-100 text-gray-800',
      cancelled: 'bg-red-100 text-red-800',
      archived: 'bg-gray-100 text-gray-600'
    }
    return colors[status] || colors.draft
  }

  const getApprovalStatusColor = (status: Event['approval_status']) => {
    const colors = {
      pending: 'bg-yellow-100 text-yellow-800',
      approved: 'bg-green-100 text-green-800',
      rejected: 'bg-red-100 text-red-800'
    }
    return colors[status] || colors.pending
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('zh-TW', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const formatPrice = (price: number, currency: string = 'TWD') => {
    return new Intl.NumberFormat('zh-TW', {
      style: 'currency',
      currency: currency
    }).format(price)
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
                onClick={() => navigate('/admin')}
                className="inline-flex items-center text-sm text-gray-500 hover:text-gray-700"
              >
                <ArrowLeft className="h-4 w-4 mr-1" />
                返回管理後台
              </button>
            </div>
            <h1 className="text-2xl font-semibold text-gray-900">活動管理</h1>
            <p className="mt-2 text-sm text-gray-700">
              為 HeSocial 平台管理頂級社交活動、場地和類別。
            </p>
          </div>
          <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none space-x-3">
            <button
              type="button"
              onClick={() => navigate('/events/venues')}
              className="inline-flex items-center justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            >
              <MapPin className="h-4 w-4 mr-2" />
              管理場地
            </button>
            <button
              type="button"
              onClick={() => navigate('/events/categories')}
              className="inline-flex items-center justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
            >
              <Users className="h-4 w-4 mr-2" />
              管理類別
            </button>
            <button
              type="button"
              onClick={() => setShowEventModal(true)}
              className="inline-flex items-center justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 sm:w-auto"
            >
              <Plus className="h-4 w-4 mr-2" />
              建立活動
            </button>
          </div>
        </div>

        {error && (
          <div className="mt-4 rounded-md bg-red-50 p-4">
            <div className="flex">
              <XCircle className="h-5 w-5 text-red-400" />
              <div className="ml-3">
                <h3 className="text-sm font-medium text-red-800">錯誤</h3>
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
                搜尋活動
              </label>
              <div className="mt-1 relative">
                <input
                  type="text"
                  id="search"
                  placeholder="依標題或描述搜尋..."
                  value={filters.search || ''}
                  onChange={(e) => handleSearch(e.target.value)}
                  className="block w-full pr-10 border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                />
                <Search className="absolute right-3 top-2.5 h-4 w-4 text-gray-400" />
              </div>
            </div>

            <div>
              <label htmlFor="status" className="block text-sm font-medium text-gray-700">
                狀態
              </label>
              <select
                id="status"
                value={filters.status || ''}
                onChange={(e) => handleFilterChange('status', e.target.value)}
                className="mt-1 block w-full border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              >
                <option value="">所有狀態</option>
                <option value="draft">草稿</option>
                <option value="pending_review">待審核</option>
                <option value="approved">已核准</option>
                <option value="published">已發布</option>
                <option value="full">已額滿</option>
                <option value="completed">已完成</option>
                <option value="cancelled">已取消</option>
                <option value="archived">已封存</option>
              </select>
            </div>

            <div>
              <label htmlFor="category" className="block text-sm font-medium text-gray-700">
                類別
              </label>
              <select
                id="category"
                value={filters.category || ''}
                onChange={(e) => handleFilterChange('category', e.target.value)}
                className="mt-1 block w-full border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              >
                <option value="">所有類別</option>
                {categories.map((category) => (
                  <option key={category.id} value={category.slug}>
                    {category.name}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label htmlFor="venue" className="block text-sm font-medium text-gray-700">
                Venue
              </label>
              <select
                id="venue"
                value={filters.venue || ''}
                onChange={(e) => handleFilterChange('venue', e.target.value)}
                className="mt-1 block w-full border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              >
                <option value="">All Venues</option>
                {venues.map((venue) => (
                  <option key={venue.id} value={venue.id}>
                    {venue.name}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {/* Events List */}
        <div className="mt-6 bg-white shadow overflow-hidden sm:rounded-md">
          {loading ? (
            <div className="p-6 text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
              <p className="mt-2 text-sm text-gray-500">Loading events...</p>
            </div>
          ) : events.length === 0 ? (
            <div className="p-6 text-center">
              <Calendar className="h-12 w-12 text-gray-400 mx-auto" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">No events found</h3>
              <p className="mt-1 text-sm text-gray-500">
                Get started by creating your first luxury social event.
              </p>
              <div className="mt-6">
                <button
                  type="button"
                  onClick={() => setShowEventModal(true)}
                  className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Create Event
                </button>
              </div>
            </div>
          ) : (
            <ul className="divide-y divide-gray-200">
              {events.map((event) => (
                <li key={event.id}>
                  <div className="px-4 py-4 sm:px-6">
                    <div className="flex items-center justify-between">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center space-x-3">
                          <h3 className="text-lg font-medium text-gray-900 truncate">
                            {event.title}
                          </h3>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(event.status)}`}>
                            {event.status.replace('_', ' ')}
                          </span>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getApprovalStatusColor(event.approval_status)}`}>
                            {event.approval_status}
                          </span>
                        </div>
                        
                        <div className="mt-2 flex items-center text-sm text-gray-500 space-x-6">
                          <div className="flex items-center">
                            <Calendar className="h-4 w-4 mr-1" />
                            {formatDate(event.start_datetime)}
                          </div>
                          {event.venue_name && (
                            <div className="flex items-center">
                              <MapPin className="h-4 w-4 mr-1" />
                              {event.venue_name}
                            </div>
                          )}
                          <div className="flex items-center">
                            <Users className="h-4 w-4 mr-1" />
                            {event.current_registrations || 0}/{event.capacity_max}
                          </div>
                          {event.category_name && (
                            <div className="flex items-center">
                              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                                {event.category_name}
                              </span>
                            </div>
                          )}
                        </div>

                        <div className="mt-2 flex items-center text-sm text-gray-500 space-x-4">
                          <span>Platinum: {formatPrice(event.price_platinum, event.currency)}</span>
                          <span>Diamond: {formatPrice(event.price_diamond, event.currency)}</span>
                          <span>Black Card: {formatPrice(event.price_black_card, event.currency)}</span>
                        </div>

                        {/* Compact Media Preview */}
                        <div className="mt-3">
                          <div className="flex items-center text-xs text-gray-400 mb-2">
                            <Image className="h-3 w-3 mr-1" />
                            Event Media
                          </div>
                          <div className="max-w-2xl">
                            <MediaGallery
                              eventId={event.id}
                              type="image"
                              editable={false}
                              className="max-h-20 overflow-hidden"
                            />
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center space-x-2">
                        {event.approval_status === 'pending' && (
                          <>
                            <button
                              onClick={() => handleApproveEvent(event.id, true)}
                              disabled={actionLoading === event.id}
                              className="inline-flex items-center px-3 py-1 border border-transparent text-xs font-medium rounded-md text-green-700 bg-green-100 hover:bg-green-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
                            >
                              <CheckCircle className="h-3 w-3 mr-1" />
                              Approve
                            </button>
                            <button
                              onClick={() => handleApproveEvent(event.id, false)}
                              disabled={actionLoading === event.id}
                              className="inline-flex items-center px-3 py-1 border border-transparent text-xs font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50"
                            >
                              <XCircle className="h-3 w-3 mr-1" />
                              Reject
                            </button>
                          </>
                        )}

                        {event.approval_status === 'approved' && event.status !== 'published' && (
                          <button
                            onClick={() => handlePublishEvent(event.id)}
                            disabled={actionLoading === event.id}
                            className="inline-flex items-center px-3 py-1 border border-transparent text-xs font-medium rounded-md text-blue-700 bg-blue-100 hover:bg-blue-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
                          >
                            <ExternalLink className="h-3 w-3 mr-1" />
                            Publish
                          </button>
                        )}

                        <button
                          onClick={() => setSelectedEvent(event)}
                          className="inline-flex items-center px-3 py-1 border border-gray-300 text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        >
                          <Eye className="h-3 w-3 mr-1" />
                          View
                        </button>

                        <button
                          onClick={() => {
                            setSelectedEvent(event)
                            setShowEventModal(true)
                          }}
                          className="inline-flex items-center px-3 py-1 border border-gray-300 text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        >
                          <Edit className="h-3 w-3 mr-1" />
                          Edit
                        </button>

                        <button
                          onClick={() => navigate(`/events/${event.id}/media`)}
                          className="inline-flex items-center px-3 py-1 border border-gray-300 text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        >
                          <Image className="h-3 w-3 mr-1" />
                          Media
                        </button>

                        {user?.role === 'super_admin' && (
                          <button
                            onClick={() => {
                              setSelectedEvent(event)
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
        {showDeleteModal && selectedEvent && (
          <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div className="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
              <div className="mt-3 text-center">
                <div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100">
                  <Trash2 className="h-6 w-6 text-red-600" />
                </div>
                <h3 className="text-lg font-medium text-gray-900 mt-4">Delete Event</h3>
                <div className="mt-2 px-7 py-3">
                  <p className="text-sm text-gray-500">
                    Are you sure you want to delete "{selectedEvent.title}"? This action cannot be undone.
                  </p>
                </div>
                <div className="items-center px-4 py-3">
                  <button
                    onClick={handleDeleteEvent}
                    disabled={actionLoading === selectedEvent.id}
                    className="px-4 py-2 bg-red-500 text-white text-base font-medium rounded-md w-full shadow-sm hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-300 disabled:opacity-50"
                  >
                    {actionLoading === selectedEvent.id ? 'Deleting...' : 'Delete Event'}
                  </button>
                  <button
                    onClick={() => {
                      setShowDeleteModal(false)
                      setSelectedEvent(null)
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

        {/* Event Create/Edit Form */}
        <EventForm
          event={selectedEvent}
          isOpen={showEventModal}
          onClose={() => {
            setShowEventModal(false)
            setSelectedEvent(null)
          }}
          onSuccess={() => {
            loadData()
          }}
        />
      </div>
    </div>
  )
}

export default EventManagement