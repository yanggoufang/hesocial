import React, { useState, useEffect } from 'react';
import { useAuth } from '../hooks/useAuth';
import { Search, Plus, Edit, Trash2 } from 'lucide-react';
import EventManagementLayout from '../components/EventManagementLayout';

// Simplified interface for this component
interface Venue {
  id: string;
  name: string;
  city: string;
  venue_type: string;
  is_active: boolean;
}

const VenueManagement: React.FC = () => {
  const { user } = useAuth();
  const [venues, setVenues] = useState<Venue[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [selectedVenue, setSelectedVenue] = useState<Venue | null>(null);
  const [newVenueName, setNewVenueName] = useState('');

  useEffect(() => {
    fetchVenues();
  }, []);

  const fetchVenues = async () => {
    setLoading(true);
    try {
      // Replace with your actual API service call
      const response = await fetch('/api/venues', { headers: { 'Authorization': `Bearer ${localStorage.getItem('token')}` } });
      const data = await response.json();
      if (data.success) {
        setVenues(data.data || []);
      } else {
        setError(data.error || 'Failed to fetch venues');
      }
    } catch (err) {
      setError('Network error');
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async () => {
    // Replace with your actual API service call
    console.log('Creating venue:', newVenueName);
    setShowCreateModal(false);
    setNewVenueName('');
    fetchVenues();
  };

  const handleDelete = async () => {
    if (!selectedVenue) return;
    // Replace with your actual API service call
    console.log('Deleting venue:', selectedVenue.id);
    setShowDeleteModal(false);
    fetchVenues();
  };

  const filteredVenues = venues.filter(v => v.name.toLowerCase().includes(searchTerm.toLowerCase()));

  if (!user || !['admin', 'super_admin'].includes(user.role)) {
    return <div>Access Denied</div>;
  }

  return (
    <EventManagementLayout>
      <div className="space-y-6">
        <div className="luxury-glass p-4 rounded-lg">
          <div className="flex flex-col md:flex-row justify-between items-center mb-4">
            <h2 className="text-xl font-bold text-luxury-platinum mb-4 md:mb-0">Venues</h2>
            <div className="flex items-center gap-4 w-full md:w-auto">
              <div className="relative flex-grow">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
                <input
                  type="text"
                  placeholder="Search venues..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="luxury-input w-full pl-10"
                />
              </div>
              <button onClick={() => setShowCreateModal(true)} className="luxury-button-primary">
                <Plus className="h-4 w-4 mr-2" />
                New Venue
              </button>
            </div>
          </div>
        </div>

        {error && <div className="bg-red-900/50 text-red-200 border border-red-700 p-4 rounded-lg">{error}</div>}

        <div className="luxury-glass overflow-x-auto rounded-lg">
          {loading ? (
            <div className="p-6 text-center text-luxury-platinum/70">Loading...</div>
          ) : (
            <table className="w-full text-sm text-left text-luxury-platinum/80">
              <thead className="text-xs text-luxury-gold uppercase bg-luxury-gold/10">
                <tr>
                  <th scope="col" className="px-6 py-3">Name</th>
                  <th scope="col" className="px-6 py-3">City</th>
                  <th scope="col" className="px-6 py-3">Type</th>
                  <th scope="col" className="px-6 py-3">Status</th>
                  <th scope="col" className="px-6 py-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredVenues.map((venue) => (
                  <tr key={venue.id} className="border-b border-luxury-gold/10 hover:bg-luxury-gold/5">
                    <td className="px-6 py-4 font-medium text-white">{venue.name}</td>
                    <td className="px-6 py-4">{venue.city}</td>
                    <td className="px-6 py-4">{venue.venue_type}</td>
                    <td className="px-6 py-4">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${venue.is_active ? 'bg-green-500/20 text-green-300' : 'bg-gray-500/20 text-gray-300'}`}>
                        {venue.is_active ? 'Active' : 'Inactive'}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right space-x-2">
                      <button className="luxury-button-icon"><Edit className="h-4 w-4" /></button>
                      <button onClick={() => { setSelectedVenue(venue); setShowDeleteModal(true); }} className="luxury-button-icon-danger"><Trash2 className="h-4 w-4" /></button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Modals */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
          <div className="luxury-glass p-6 rounded-lg text-center w-full max-w-md">
            <h3 className="text-lg font-bold text-luxury-gold mb-4">Create New Venue</h3>
            <input
              type="text"
              placeholder="Venue Name"
              value={newVenueName}
              onChange={(e) => setNewVenueName(e.target.value)}
              className="luxury-input w-full mb-4"
            />
            <div className="mt-6 space-x-4">
              <button onClick={() => setShowCreateModal(false)} className="luxury-button-secondary">Cancel</button>
              <button onClick={handleCreate} className="luxury-button-primary">Create</button>
            </div>
          </div>
        </div>
      )}
      {showDeleteModal && selectedVenue && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
          <div className="luxury-glass p-6 rounded-lg text-center">
            <h3 className="text-lg font-bold text-luxury-gold mb-4">Delete Venue</h3>
            <p>Are you sure you want to delete "{selectedVenue.name}"?</p>
            <div className="mt-6 space-x-4">
              <button onClick={() => setShowDeleteModal(false)} className="luxury-button-secondary">Cancel</button>
              <button onClick={handleDelete} className="luxury-button-danger">Delete</button>
            </div>
          </div>
        </div>
      )}
    </EventManagementLayout>
  );
};

export default VenueManagement;
