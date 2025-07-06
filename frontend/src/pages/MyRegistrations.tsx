import React, { useState, useEffect } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import registrationService, { Registration } from '../services/registrationService'
import { 
  Calendar, 
  MapPin, 
  Clock, 
  Eye, 
  Edit, 
  X, 
  Check, 
  AlertTriangle, 
  CreditCard,
  Filter,
  Search,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  ExternalLink
} from 'lucide-react'

const MyRegistrations: React.FC = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const { user, isAuthenticated } = useAuth()
  
  const [registrations, setRegistrations] = useState<Registration[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  
  // Filters and pagination
  const [filters, setFilters] = useState({
    status: '',
    paymentStatus: '',
    search: ''
  })
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(1)
  const [totalRegistrations, setTotalRegistrations] = useState(0)
  const pageSize = 10

  // Modal states
  const [selectedRegistration, setSelectedRegistration] = useState<Registration | null>(null)
  const [showEditModal, setShowEditModal] = useState(false)
  const [editRequests, setEditRequests] = useState('')
  const [actionLoading, setActionLoading] = useState<string | null>(null)

  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login')
      return
    }

    // Check for success message from navigation state
    if (location.state?.message) {
      setSuccessMessage(location.state.message)
      // Clear the state
      window.history.replaceState({}, document.title)
    }

    fetchRegistrations()
  }, [isAuthenticated, navigate, currentPage, filters])

  const fetchRegistrations = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await registrationService.getUserRegistrations({
        ...filters,
        page: currentPage,
        limit: pageSize
      })

      if (response.success && response.data) {
        setRegistrations(response.data)
        if (response.pagination) {
          setTotalPages(response.pagination.totalPages)
          setTotalRegistrations(response.pagination.total)
        }
      } else {
        setError(response.error || 'Failed to fetch registrations')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const handleFilterChange = (key: string, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }))
    setCurrentPage(1) // Reset to first page when filtering
  }

  const clearFilters = () => {
    setFilters({
      status: '',
      paymentStatus: '',
      search: ''
    })
    setCurrentPage(1)
  }

  const handleEditRegistration = async () => {
    if (!selectedRegistration) return
    
    setActionLoading('edit')
    
    try {
      const response = await registrationService.updateRegistration(
        selectedRegistration.id,
        { specialRequests: editRequests }
      )
      
      if (response.success) {
        setShowEditModal(false)
        fetchRegistrations()
        setSuccessMessage('Registration updated successfully')
      } else {
        setError(response.error || 'Failed to update registration')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const handleCancelRegistration = async (registrationId: string) => {
    if (!confirm('Are you sure you want to cancel this registration? This action cannot be undone.')) {
      return
    }
    
    setActionLoading(`cancel-${registrationId}`)
    
    try {
      const response = await registrationService.cancelRegistration(registrationId)
      
      if (response.success) {
        fetchRegistrations()
        setSuccessMessage('Registration cancelled successfully')
      } else {
        setError(response.error || 'Failed to cancel registration')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setActionLoading(null)
    }
  }

  const openEditModal = (registration: Registration) => {
    setSelectedRegistration(registration)
    setEditRequests(registration.specialRequests || '')
    setShowEditModal(true)
  }

  const getStatusBadge = (status: string) => {
    const styles = {
      'pending': 'bg-yellow-100 text-yellow-800 border-yellow-300',
      'approved': 'bg-green-100 text-green-800 border-green-300',
      'rejected': 'bg-red-100 text-red-800 border-red-300',
      'cancelled': 'bg-gray-100 text-gray-800 border-gray-300'
    }
    
    const icons = {
      'pending': <Clock className="w-3 h-3" />,
      'approved': <Check className="w-3 h-3" />,
      'rejected': <X className="w-3 h-3" />,
      'cancelled': <AlertTriangle className="w-3 h-3" />
    }

    return (
      <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border ${styles[status as keyof typeof styles]}`}>
        {icons[status as keyof typeof icons]}
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </span>
    )
  }

  const getPaymentBadge = (status: string) => {
    const styles = {
      'pending': 'bg-yellow-100 text-yellow-800',
      'paid': 'bg-green-100 text-green-800',
      'refunded': 'bg-blue-100 text-blue-800'
    }

    return (
      <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium ${styles[status as keyof typeof styles]}`}>
        <CreditCard className="w-3 h-3" />
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </span>
    )
  }

  const formatDateTime = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('zh-TW', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const canEdit = (registration: Registration) => {
    return registration.status === 'pending' && 
           new Date(registration.eventDateTime) > new Date()
  }

  const canCancel = (registration: Registration) => {
    const eventDate = new Date(registration.eventDateTime)
    const now = new Date()
    const hoursUntilEvent = (eventDate.getTime() - now.getTime()) / (1000 * 3600)
    
    return registration.status !== 'cancelled' && 
           registration.status !== 'rejected' &&
           hoursUntilEvent > 24
  }

  if (!isAuthenticated) {
    return null
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">My Event Registrations</h1>
              <p className="text-gray-600 mt-2">Manage your event registrations and applications</p>
            </div>
            <button
              onClick={() => navigate('/events')}
              className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
            >
              <ExternalLink className="w-4 h-4" />
              Browse Events
            </button>
          </div>
        </div>

        {/* Success Message */}
        {successMessage && (
          <div className="mb-6 bg-green-50 border border-green-200 rounded-lg p-4">
            <div className="flex items-center">
              <Check className="w-5 h-5 text-green-600 mr-2" />
              <p className="text-green-800">{successMessage}</p>
              <button
                onClick={() => setSuccessMessage(null)}
                className="ml-auto text-green-600 hover:text-green-800"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Error Message */}
        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
            <div className="flex items-center">
              <X className="w-5 h-5 text-red-600 mr-2" />
              <p className="text-red-800">{error}</p>
              <button
                onClick={() => setError(null)}
                className="ml-auto text-red-600 hover:text-red-800"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Filters */}
        <div className="bg-white rounded-lg shadow-sm border mb-6 p-4">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-medium text-gray-900">Filters</h3>
            <button
              onClick={fetchRegistrations}
              className="inline-flex items-center gap-2 px-3 py-1 text-sm text-gray-600 hover:text-gray-900"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Search</label>
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search events..."
                  value={filters.search}
                  onChange={(e) => handleFilterChange('search', e.target.value)}
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                />
              </div>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Status</label>
              <select
                value={filters.status}
                onChange={(e) => handleFilterChange('status', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              >
                <option value="">All Statuses</option>
                <option value="pending">Pending</option>
                <option value="approved">Approved</option>
                <option value="rejected">Rejected</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Payment Status</label>
              <select
                value={filters.paymentStatus}
                onChange={(e) => handleFilterChange('paymentStatus', e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              >
                <option value="">All Payment Statuses</option>
                <option value="pending">Pending</option>
                <option value="paid">Paid</option>
                <option value="refunded">Refunded</option>
              </select>
            </div>
            
            <div className="flex items-end">
              <button
                onClick={clearFilters}
                className="w-full px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Clear Filters
              </button>
            </div>
          </div>
        </div>

        {/* Registrations List */}
        <div className="bg-white rounded-lg shadow-sm border overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">
              Registrations ({totalRegistrations} total)
            </h3>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-12">
              <RefreshCw className="w-6 h-6 text-gray-400 animate-spin" />
              <span className="ml-2 text-gray-600">Loading registrations...</span>
            </div>
          ) : registrations.length === 0 ? (
            <div className="text-center py-12">
              <Calendar className="w-16 h-16 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No registrations found</h3>
              <p className="text-gray-600 mb-4">You haven't registered for any events yet.</p>
              <button
                onClick={() => navigate('/events')}
                className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
                Browse Events
              </button>
            </div>
          ) : (
            <div className="divide-y divide-gray-200">
              {registrations.map((registration) => (
                <div key={registration.id} className="p-6 hover:bg-gray-50">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-start gap-4">
                        <div className="flex-1">
                          <h4 className="text-lg font-medium text-gray-900 mb-2">
                            {registration.eventName}
                          </h4>
                          
                          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-3">
                            <div className="flex items-center gap-2 text-sm text-gray-600">
                              <Calendar className="w-4 h-4" />
                              <span>{formatDateTime(registration.eventDateTime)}</span>
                            </div>
                            <div className="flex items-center gap-2 text-sm text-gray-600">
                              <MapPin className="w-4 h-4" />
                              <span>{registration.venueName}</span>
                            </div>
                            <div className="flex items-center gap-2 text-sm text-gray-600">
                              <Clock className="w-4 h-4" />
                              <span>Registered {formatDateTime(registration.createdAt)}</span>
                            </div>
                          </div>
                          
                          <div className="flex items-center gap-3 mb-3">
                            {getStatusBadge(registration.status)}
                            {getPaymentBadge(registration.paymentStatus)}
                          </div>
                          
                          {registration.specialRequests && (
                            <div className="bg-gray-50 rounded-lg p-3 mb-3">
                              <p className="text-sm font-medium text-gray-700 mb-1">Special Requests:</p>
                              <p className="text-sm text-gray-600">{registration.specialRequests}</p>
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2 ml-4">
                      <button
                        onClick={() => navigate(`/events/${registration.eventId}`)}
                        className="p-2 text-gray-400 hover:text-gray-600"
                        title="View Event"
                      >
                        <Eye className="w-4 h-4" />
                      </button>
                      
                      {canEdit(registration) && (
                        <button
                          onClick={() => openEditModal(registration)}
                          className="p-2 text-blue-400 hover:text-blue-600"
                          title="Edit Registration"
                        >
                          <Edit className="w-4 h-4" />
                        </button>
                      )}
                      
                      {canCancel(registration) && (
                        <button
                          onClick={() => handleCancelRegistration(registration.id)}
                          disabled={actionLoading === `cancel-${registration.id}`}
                          className="p-2 text-red-400 hover:text-red-600 disabled:opacity-50"
                          title="Cancel Registration"
                        >
                          <X className="w-4 h-4" />
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="px-6 py-4 border-t border-gray-200 flex items-center justify-between">
              <div className="text-sm text-gray-700">
                Showing {((currentPage - 1) * pageSize) + 1} to {Math.min(currentPage * pageSize, totalRegistrations)} of {totalRegistrations} results
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
                  disabled={currentPage === 1}
                  className="inline-flex items-center px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <ChevronLeft className="w-4 h-4" />
                  Previous
                </button>
                <span className="text-sm text-gray-700">
                  Page {currentPage} of {totalPages}
                </span>
                <button
                  onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                  disabled={currentPage === totalPages}
                  className="inline-flex items-center px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Next
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Edit Registration Modal */}
      {showEditModal && selectedRegistration && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-bold text-gray-900">Edit Registration</h3>
              <button
                onClick={() => setShowEditModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-6 h-6" />
              </button>
            </div>
            
            <div className="mb-4">
              <p className="text-sm text-gray-600 mb-3">
                Event: <strong>{selectedRegistration.eventName}</strong>
              </p>
              
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Special Requests
              </label>
              <textarea
                value={editRequests}
                onChange={(e) => setEditRequests(e.target.value)}
                rows={4}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                placeholder="Any dietary restrictions, accessibility needs, or special requests..."
              />
            </div>
            
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowEditModal(false)}
                className="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleEditRegistration}
                disabled={actionLoading === 'edit'}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2"
              >
                {actionLoading === 'edit' && <RefreshCw className="w-4 h-4 animate-spin" />}
                Save Changes
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default MyRegistrations