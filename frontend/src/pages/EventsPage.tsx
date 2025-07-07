import { useState, useEffect, useCallback } from 'react';
import { motion } from 'framer-motion';
import { Link } from 'react-router-dom';
import { Calendar, MapPin, Users, Star, Search, ChevronLeft, ChevronRight, Diamond } from 'lucide-react';
import axios from 'axios';

// Define the types for our data structures
interface Venue {
  name: string;
  address: string;
  rating: number;
}

interface Pricing {
  vvip?: number;
  vip?: number;
  currency: string;
}

interface Event {
  id: string;
  name: string;
  description: string;
  dateTime: string;
  venue: Venue;
  category: string;
  exclusivityLevel: string;
  pricing: Pricing;
  currentAttendees: number;
  capacity: number;
  dressCode: number;
  images: string[] | null;
  tags: string[];
}

interface Pagination {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
}

const EventsPage = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [selectedLevel, setSelectedLevel] = useState('all');
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);
  const [pagination, setPagination] = useState<Pagination>({
    page: 1,
    limit: 9, // Changed: Display 9 events per page
    total: 0,
    totalPages: 1,
  });

  const categories = [
    { id: 'all', name: '全部活動' },
    { id: 'dinner', name: '私人晚宴' },
    { id: 'yacht', name: '遊艇派對' },
    { id: 'art', name: '藝術沙龍' },
    { id: 'business', name: '商務社交' },
    { id: 'wine', name: '品酒會' },
  ];

  const exclusivityLevels = [
    { id: 'all', name: '全部等級' },
    { id: 'VIP', name: 'VIP' },
    { id: 'VVIP', name: 'VVIP' },
    { id: 'invitation', name: '僅限邀請' },
  ];

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-TW', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long',
    });
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleTimeString('zh-TW', {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getExclusivityColor = (level: string) => {
    switch (level) {
      case 'VIP':
        return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
      case 'VVIP':
        return 'bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30';
      case '僅限邀請':
        return 'bg-purple-500/20 text-purple-400 border-purple-500/30';
      default:
        return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
    }
  };

  const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api';

  const fetchEvents = useCallback(async (page: number) => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        limit: pagination.limit.toString(),
      });

      if (searchTerm) params.append('search', searchTerm);
      if (selectedCategory !== 'all') params.append('category', selectedCategory);
      if (selectedLevel !== 'all') params.append('exclusivityLevel', selectedLevel);

      const response = await axios.get<{ success: boolean; data: Event[]; pagination: Pagination }>(`${API_BASE_URL}/events?${params}`);
      
      if (response.data.success) {
        setEvents(response.data.data);
        setPagination(response.data.pagination);
      } else {
        // Handle API error gracefully
        setEvents([]);
        setPagination(prev => ({ ...prev, total: 0, totalPages: 1, page }));
      }
    } catch (error) {
      console.error('Failed to fetch events:', error);
      setEvents([]); // Clear events on error
      setPagination(prev => ({ ...prev, total: 0, totalPages: 1, page }));
    } finally {
      setLoading(false);
    }
  }, [pagination.limit, searchTerm, selectedCategory, selectedLevel]);

  // Reset to page 1 when filters change
  useEffect(() => {
    fetchEvents(1);
  }, [searchTerm, selectedCategory, selectedLevel, fetchEvents]);

  const handlePageChange = (newPage: number) => {
    if (newPage >= 1 && newPage <= pagination.totalPages) {
      fetchEvents(newPage);
    }
  };

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="text-center mb-12"
        >
          <h1 className="text-4xl md:text-6xl font-luxury font-bold text-luxury-gold mb-6">
            精選活動
          </h1>
          <p className="text-xl text-luxury-platinum/80 max-w-3xl mx-auto">
            探索為您精心策劃的頂級社交活動，與志同道合的菁英建立深度連結
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="luxury-glass p-6 rounded-2xl mb-8"
        >
          {/* Filter controls... */}
        </motion.div>

        {loading ? (
          <div className="text-center text-luxury-gold text-2xl">載入中...</div>
        ) : events.length > 0 ? (
          <>
            <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-8">
              {events.map((event, index) => (
                <motion.div
                  key={event.id}
                  initial={{ opacity: 0, y: 50 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.8, delay: index * 0.1 }}
                  className="relative luxury-glass rounded-2xl hover:bg-luxury-gold/5 transition-all duration-300 group"
                >
                  <div className="relative">
                    <div className="rounded-t-2xl overflow-hidden">
                      <img
                        src={event.images && event.images.length > 0 ? event.images[0] : '/api/placeholder/400/300'}
                        alt={event.name}
                        className="w-full h-48 object-cover group-hover:scale-105 transition-transform duration-300"
                      />
                    </div>
                    <div className="absolute top-4 left-4">
                      <span className={`px-3 py-1 rounded-full text-xs font-medium border ${getExclusivityColor(event.exclusivityLevel)}`}>
                        {event.exclusivityLevel}
                      </span>
                    </div>
                    <div className="absolute top-4 right-4 flex items-center space-x-1">
                      {event.exclusivityLevel === 'Invitation Only' && (
                        <Diamond className="h-4 w-4 text-white fill-current mr-1" />
                      )}

                      {(event.exclusivityLevel === 'VVIP' || event.exclusivityLevel === 'Invitation Only') && (
                        <>
                          <Star className="h-4 w-4 text-luxury-gold fill-current" />
                          <Star className="h-4 w-4 text-luxury-gold fill-current" />
                          <Star className="h-4 w-4 text-luxury-gold fill-current" />
                        </>
                      )}

                      {event.exclusivityLevel === 'VIP' && (
                        <>
                          <Star className="h-4 w-4 text-luxury-gold fill-current" />
                          <Star className="h-4 w-4 text-luxury-gold fill-current" />
                        </>
                      )}
                    </div>
                  </div>

                  <div className="p-6">
                    <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-2 group-hover:text-luxury-gold/90 transition-colors">
                      {event.name}
                    </h3>
                    <p className="text-luxury-platinum/80 text-sm mb-4 line-clamp-2">
                      {event.description}
                    </p>

                    <div className="space-y-3 mb-6">
                      <div className="flex items-center text-luxury-platinum/70 text-sm">
                        <Calendar className="h-4 w-4 mr-2 text-luxury-gold" />
                        <span>{formatDate(event.dateTime)} {formatTime(event.dateTime)}</span>
                      </div>
                      <div className="flex items-center text-luxury-platinum/70 text-sm">
                        <MapPin className="h-4 w-4 mr-2 text-luxury-gold" />
                        <span>{event.venue.name}</span>
                      </div>
                      <div className="flex items-center text-luxury-platinum/70 text-sm">
                        <Users className="h-4 w-4 mr-2 text-luxury-gold" />
                        <span>{event.currentAttendees}/{event.capacity} 人</span>
                      </div>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="text-luxury-gold font-semibold">
                        {event.pricing.vvip ? `NT$ ${event.pricing.vvip.toLocaleString()}` : 
                         event.pricing.vip ? `NT$ ${event.pricing.vip.toLocaleString()}` : '價格洽詢'}
                      </div>
                      <Link
                        to={`/events/${event.id}`}
                        className="px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold hover:text-luxury-midnight-black transition-all duration-300 text-sm font-medium"
                      >
                        查看詳情
                      </Link>
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>

            {/* Pagination Controls */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.8, delay: 0.4 }}
              className="flex items-center justify-center mt-12 space-x-4 text-luxury-platinum"
            >
              <button
                onClick={() => handlePageChange(pagination.page - 1)}
                disabled={pagination.page <= 1 || loading}
                className="luxury-button-outline p-2 rounded-full disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ChevronLeft className="h-5 w-5" />
              </button>
              
              <span className="font-semibold">
                第 {pagination.page} / {pagination.totalPages} 頁
              </span>

              <button
                onClick={() => handlePageChange(pagination.page + 1)}
                disabled={pagination.page >= pagination.totalPages || loading}
                className="luxury-button-outline p-2 rounded-full disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <ChevronRight className="h-5 w-5" />
              </button>
            </motion.div>
          </>
        ) : (
          <div className="text-center text-luxury-platinum/80 text-xl py-20">
            <p>找不到符合條件的活動。</p>
            <p>請嘗試調整您的篩選條件，或稍後再試。</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default EventsPage;
