import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { Link } from 'react-router-dom'
import { Calendar, MapPin, Users, Star, Search } from 'lucide-react'
import axios from 'axios'

const EventsPage = () => {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [selectedLevel, setSelectedLevel] = useState('all')
  const [events, setEvents] = useState<any[]>([])
  const [loading, setLoading] = useState(false)
  const [pagination, setPagination] = useState({
    page: 1,
    limit: 2,
    total: 0,
    totalPages: 0
  })
  const [hasMore, setHasMore] = useState(true)

  const categories = [
    { id: 'all', name: '全部活動' },
    { id: 'dinner', name: '私人晚宴' },
    { id: 'yacht', name: '遊艇派對' },
    { id: 'art', name: '藝術沙龍' },
    { id: 'business', name: '商務社交' },
    { id: 'wine', name: '品酒會' }
  ]

  const exclusivityLevels = [
    { id: 'all', name: '全部等級' },
    { id: 'VIP', name: 'VIP' },
    { id: 'VVIP', name: 'VVIP' },
    { id: 'invitation', name: '僅限邀請' }
  ]

  const mockEvents = [
    {
      id: '1',
      name: '星空下的法式晚宴',
      description: '與米其林三星主廚共享精緻法式料理，在台北101頂樓欣賞城市夜景',
      dateTime: '2024-07-27T19:00:00',
      venue: {
        name: '台北文華東方酒店',
        address: '台北市松山區敦化北路166號',
        rating: 5
      },
      category: '私人晚宴',
      exclusivityLevel: 'VVIP',
      pricing: {
        vvip: 15000,
        vip: 10000,
        currency: 'TWD'
      },
      currentAttendees: 12,
      capacity: 20,
      dressCode: 5,
      images: ['/api/placeholder/400/300'],
      tags: ['米其林', '頂樓', '限量']
    },
    {
      id: '2',
      name: '私人遊艇品酒之夜',
      description: '在豪華遊艇上品嚐世界頂級香檳，與成功企業家建立深度連結',
      dateTime: '2024-08-15T18:30:00',
      venue: {
        name: '基隆港VIP碼頭',
        address: '基隆市中正區中正路1號',
        rating: 5
      },
      category: '遊艇派對',
      exclusivityLevel: 'VIP',
      pricing: {
        vip: 8000,
        currency: 'TWD'
      },
      currentAttendees: 8,
      capacity: 16,
      dressCode: 4,
      images: ['/api/placeholder/400/300'],
      tags: ['遊艇', '香檳', '夕陽']
    },
    {
      id: '3',
      name: '當代藝術收藏家沙龍',
      description: '與知名藝術策展人探討當代藝術趨勢，欣賞私人收藏珍品',
      dateTime: '2024-09-08T15:00:00',
      venue: {
        name: '台北當代藝術館',
        address: '台北市大同區長安西路39號',
        rating: 4
      },
      category: '藝術沙龍',
      exclusivityLevel: '僅限邀請',
      pricing: {
        vvip: 12000,
        currency: 'TWD'
      },
      currentAttendees: 6,
      capacity: 12,
      dressCode: 3,
      images: ['/api/placeholder/400/300'],
      tags: ['藝術', '收藏', '策展人']
    }
  ]

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-TW', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long'
    })
  }

  const formatTime = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleTimeString('zh-TW', {
      hour: '2-digit',
      minute: '2-digit'
    })
  }


  const getExclusivityColor = (level: string) => {
    switch (level) {
      case 'VIP':
        return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
      case 'VVIP':
        return 'bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30'
      case '僅限邀請':
        return 'bg-purple-500/20 text-purple-400 border-purple-500/30'
      default:
        return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
    }
  }

  // API base URL - you may need to adjust this based on your setup
  const API_BASE_URL = 'http://localhost:5000/api'

  const fetchEvents = async (reset = false) => {
    if (loading) return
    
    setLoading(true)
    try {
      const currentPage = reset ? 1 : pagination.page
      const params = new URLSearchParams({
        page: currentPage.toString(),
        limit: pagination.limit.toString(),
      })

      if (searchTerm) params.append('search', searchTerm)
      if (selectedCategory !== 'all') params.append('category', selectedCategory)
      if (selectedLevel !== 'all') params.append('exclusivityLevel', selectedLevel)

      const response = await axios.get(`${API_BASE_URL}/events?${params}`)
      
      if (response.data.success) {
        const newEvents = response.data.data
        const newPagination = response.data.pagination

        if (reset) {
          setEvents(newEvents)
        } else {
          setEvents(prev => [...prev, ...newEvents])
        }
        
        setPagination(newPagination)
        setHasMore(newPagination.page < newPagination.totalPages)
      }
    } catch (error) {
      console.error('Failed to fetch events:', error)
      // Fallback to mock data on error
      if (reset || events.length === 0) {
        setEvents(mockEvents)
        setHasMore(false)
      }
    } finally {
      setLoading(false)
    }
  }

  const loadMoreEvents = () => {
    if (hasMore && !loading) {
      setPagination(prev => ({ ...prev, page: prev.page + 1 }))
    }
  }

  // Fetch events on component mount and when filters change
  useEffect(() => {
    fetchEvents(true)
  }, [searchTerm, selectedCategory, selectedLevel])

  // Fetch more events when page changes
  useEffect(() => {
    if (pagination.page > 1) {
      fetchEvents(false)
    }
  }, [pagination.page])

  // Use API events if available, otherwise fallback to mock events
  const displayEvents = events.length > 0 ? events : mockEvents

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
          <div className="flex flex-col lg:flex-row gap-4 items-center">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
              <input
                type="text"
                placeholder="搜尋活動名稱或關鍵字..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold"
              />
            </div>

            <div className="flex flex-col sm:flex-row gap-4">
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold"
              >
                {categories.map((category) => (
                  <option key={category.id} value={category.id}>
                    {category.name}
                  </option>
                ))}
              </select>

              <select
                value={selectedLevel}
                onChange={(e) => setSelectedLevel(e.target.value)}
                className="bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold"
              >
                {exclusivityLevels.map((level) => (
                  <option key={level.id} value={level.id}>
                    {level.name}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </motion.div>

        <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-8">
          {displayEvents.map((event, index) => (
            <motion.div
              key={event.id}
              initial={{ opacity: 0, y: 50 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: index * 0.2 }}
              className="luxury-glass rounded-2xl overflow-hidden hover:bg-luxury-gold/5 transition-all duration-300 group"
            >
              <div className="relative rounded-t-2xl overflow-hidden">
                <img
                  src={event.images[0]}
                  alt={event.name}
                  className="w-full h-48 object-cover group-hover:scale-105 transition-transform duration-300"
                />
                <div className="absolute top-4 left-4">
                  <span className={`px-3 py-1 rounded-full text-xs font-medium border ${getExclusivityColor(event.exclusivityLevel)}`}>
                    {event.exclusivityLevel}
                  </span>
                </div>
                <div className="absolute top-4 right-4 flex space-x-1">
                  {Array.from({ length: event.venue.rating }).map((_, i) => (
                    <Star key={i} className="h-4 w-4 text-luxury-gold fill-current" />
                  ))}
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

                <div className="flex flex-wrap gap-2 mb-4">
                  {event.tags?.map((tag: string, tagIndex: number) => (
                    <span
                      key={tagIndex}
                      className="px-2 py-1 bg-luxury-gold/20 text-luxury-gold text-xs rounded-full"
                    >
                      {tag}
                    </span>
                  ))}
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

        {hasMore && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.8, delay: 0.6 }}
            className="text-center mt-12"
          >
            <button
              onClick={loadMoreEvents}
              disabled={loading}
              className="luxury-button-outline px-8 py-3 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? '載入中...' : '載入更多活動'}
            </button>
          </motion.div>
        )}
      </div>
    </div>
  )
}

export default EventsPage