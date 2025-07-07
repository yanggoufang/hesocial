import React, { useState, useEffect } from 'react';
import { Plus, Search, Eye, Edit, Trash2, CheckCircle, XCircle, ExternalLink, Image } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import eventService, { Event, EventFilters, EventCategory, Venue } from '../services/eventService';
import EventForm from '../components/EventForm';
import EventManagementLayout from '../components/EventManagementLayout';
import MediaGallery from '../components/MediaGallery';

const EventManagement: React.FC = () => {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [events, setEvents] = useState<Event[]>([]);
  const [categories, setCategories] = useState<EventCategory[]>([]);
  const [venues, setVenues] = useState<Venue[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<EventFilters>({ page: 1, limit: 10 });
  const [pagination, setPagination] = useState({ page: 1, limit: 10, total: 0, totalPages: 0 });
  const [selectedEvent, setSelectedEvent] = useState<Event | null>(null);
  const [showEventModal, setShowEventModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  useEffect(() => {
    if (user && ['admin', 'super_admin'].includes(user.role)) {
      loadData();
    }
  }, [user, filters]);

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [eventsResponse, categoriesResponse, venuesResponse] = await Promise.all([
        eventService.getEvents(filters),
        eventService.getCategories(),
        eventService.getVenues({ limit: 100 }),
      ]);
      if (eventsResponse.success) {
        setEvents(eventsResponse.data);
        if (eventsResponse.pagination) setPagination(eventsResponse.pagination);
      }
      if (categoriesResponse.success) setCategories(categoriesResponse.data);
      if (venuesResponse.success) setVenues(venuesResponse.data);
    } catch (err: any) {
      setError(err.message || 'Failed to load event data');
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (searchTerm: string) => {
    setFilters(prev => ({ ...prev, search: searchTerm, page: 1 }));
  };

  const handleFilterChange = (key: keyof EventFilters, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value || undefined, page: 1 }));
  };

  const handlePageChange = (newPage: number) => {
    if (newPage > 0 && newPage <= pagination.totalPages) {
      setFilters(prev => ({ ...prev, page: newPage }));
    }
  };

  const handleAction = async (action: Promise<any>) => {
    try {
      await action;
      await loadData();
    } catch (err: any) {
      setError(err.message || 'An error occurred');
    } finally {
      setActionLoading(null);
    }
  };

  const getStatusColor = (status: Event['status']) => {
    const colors = {
      draft: 'bg-gray-500/20 text-gray-300',
      pending_review: 'bg-yellow-500/20 text-yellow-300',
      approved: 'bg-blue-500/20 text-blue-300',
      published: 'bg-green-500/20 text-green-300',
      full: 'bg-purple-500/20 text-purple-300',
      completed: 'bg-gray-600/30 text-gray-400',
      cancelled: 'bg-red-500/20 text-red-300',
      archived: 'bg-gray-700/30 text-gray-500',
    };
    return colors[status] || colors.draft;
  };

  const formatDate = (dateString: string) => new Date(dateString).toLocaleString('zh-TW', { dateStyle: 'medium', timeStyle: 'short' });
  const formatPrice = (price: number, currency: string = 'TWD') => new Intl.NumberFormat('zh-TW', { style: 'currency', currency }).format(price);

  if (!user || !['admin', 'super_admin'].includes(user.role)) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center text-luxury-platinum">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Access Denied</h1>
          <p>You need admin privileges to access this page.</p>
        </div>
      </div>
    );
  }

  return (
    <EventManagementLayout>
      <div className="space-y-6">
        {/* Header and Filters */}
        <div className="luxury-glass p-4 rounded-lg">
          <div className="flex flex-col md:flex-row justify-between items-center mb-4">
            <h2 className="text-xl font-bold text-luxury-platinum mb-4 md:mb-0">Event List</h2>
            <button
              type="button"
              onClick={() => setShowEventModal(true)}
              className="luxury-button-primary w-full md:w-auto"
            >
              <Plus className="h-4 w-4 mr-2" />
              Create Event
            </button>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Search Input */}
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
              <input
                type="text"
                placeholder="Search by title..."
                value={filters.search || ''}
                onChange={(e) => handleSearch(e.target.value)}
                className="luxury-input w-full pl-10"
              />
            </div>
            {/* Other filters */}
            <select onChange={(e) => handleFilterChange('status', e.target.value)} className="luxury-input">
              <option value="">All Statuses</option>
              <option value="draft">Draft</option>
              <option value="pending_review">Pending Review</option>
              <option value="approved">Approved</option>
              <option value="published">Published</option>
            </select>
            <select onChange={(e) => handleFilterChange('category', e.target.value)} className="luxury-input">
              <option value="">All Categories</option>
              {categories.map(c => <option key={c.id} value={c.slug}>{c.name}</option>)}
            </select>
            <select onChange={(e) => handleFilterChange('venue', e.target.value)} className="luxury-input">
              <option value="">All Venues</option>
              {venues.map(v => <option key={v.id} value={v.id}>{v.name}</option>)}
            </select>
          </div>
        </div>

        {error && <div className="bg-red-900/50 text-red-200 border border-red-700 p-4 rounded-lg">{error}</div>}

        {/* Events Table */}
        <div className="luxury-glass overflow-x-auto rounded-lg">
          {loading ? (
            <div className="p-6 text-center text-luxury-platinum/70">Loading events...</div>
          ) : events.length === 0 ? (
            <div className="p-6 text-center text-luxury-platinum/70">No events found.</div>
          ) : (
            <table className="w-full text-sm text-left text-luxury-platinum/80">
              <thead className="text-xs text-luxury-gold uppercase bg-luxury-gold/10">
                <tr>
                  <th scope="col" className="px-6 py-3">Event</th>
                  <th scope="col" className="px-6 py-3">Status</th>
                  <th scope="col" className="px-6 py-3">Details</th>
                  <th scope="col" className="px-6 py-3">Pricing</th>
                  <th scope="col" className="px-6 py-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {events.map((event) => (
                  <tr key={event.id} className="border-b border-luxury-gold/10 hover:bg-luxury-gold/5">
                    <td className="px-6 py-4 font-medium text-white">{event.title}</td>
                    <td className="px-6 py-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(event.status)}`}>
                        {event.status.replace('_', ' ')}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div>{event.venue_name}</div>
                      <div className="text-xs text-luxury-platinum/60">{formatDate(event.start_datetime)}</div>
                    </td>
                    <td className="px-6 py-4">
                      <div>{formatPrice(event.price_platinum, event.currency)} (Plat)</div>
                      <div className="text-xs text-luxury-platinum/60">{formatPrice(event.price_diamond, event.currency)} (Dia)</div>
                    </td>
                    <td className="px-6 py-4 text-right space-x-2">
                      {/* Action buttons */}
                      <button onClick={() => { setSelectedEvent(event); setShowEventModal(true); }} className="luxury-button-icon"><Edit className="h-4 w-4" /></button>
                      <button onClick={() => navigate(`/events/${event.id}/media`)} className="luxury-button-icon"><Image className="h-4 w-4" /></button>
                      {user?.role === 'super_admin' && <button onClick={() => { setSelectedEvent(event); setShowDeleteModal(true); }} className="luxury-button-icon-danger"><Trash2 className="h-4 w-4" /></button>}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Pagination */}
        {pagination.totalPages > 1 && (
          <div className="flex items-center justify-between text-luxury-platinum/80">
            <span>Page {pagination.page} of {pagination.totalPages}</span>
            <div className="space-x-2">
              <button onClick={() => handlePageChange(pagination.page - 1)} disabled={pagination.page <= 1} className="luxury-button-secondary">Previous</button>
              <button onClick={() => handlePageChange(pagination.page + 1)} disabled={pagination.page >= pagination.totalPages} className="luxury-button-secondary">Next</button>
            </div>
          </div>
        )}
      </div>

      {/* Modals */}
      <EventForm event={selectedEvent} isOpen={showEventModal} onClose={() => { setShowEventModal(false); setSelectedEvent(null); }} onSuccess={loadData} />
      {showDeleteModal && selectedEvent && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
          <div className="luxury-glass p-6 rounded-lg text-center">
            <h3 className="text-lg font-bold text-luxury-gold mb-4">Delete Event</h3>
            <p>Are you sure you want to delete "{selectedEvent.title}"?</p>
            <div className="mt-6 space-x-4">
              <button onClick={() => { setShowDeleteModal(false); setSelectedEvent(null); }} className="luxury-button-secondary">Cancel</button>
              <button onClick={() => handleAction(eventService.deleteEvent(selectedEvent.id)).then(() => setShowDeleteModal(false))} className="luxury-button-danger">Delete</button>
            </div>
          </div>
        </div>
      )}
    </EventManagementLayout>
  );
};

export default EventManagement;