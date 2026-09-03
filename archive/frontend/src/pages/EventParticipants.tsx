import React, { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { motion } from 'framer-motion'
import { 
  ArrowLeft, 
  Users, 
  Search, 
  Filter, 
  Eye,
  MessageCircle,
  Crown,
  Star,
  Shield,
  Settings,
  AlertCircle,
  Lock,
  CreditCard
} from 'lucide-react'
import participantService from '../services/participantService'
import { 
  ParticipantListResponse, 
  ParticipantAccessCheckResponse,
  ParticipantFilters,
  FilteredParticipantInfo
} from '../types/participant'

const EventParticipants: React.FC = () => {
  const { eventId } = useParams<{ eventId: string }>()
  const navigate = useNavigate()
  const { user, isAuthenticated } = useAuth()

  // State management
  const [participants, setParticipants] = useState<ParticipantListResponse | null>(null)
  const [accessCheck, setAccessCheck] = useState<ParticipantAccessCheckResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [currentPage, setCurrentPage] = useState(1)
  const [filters, setFilters] = useState<ParticipantFilters>({})
  const [showFilters, setShowFilters] = useState(false)

  const pageSize = 12

  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login')
      return
    }

    if (eventId) {
      checkAccess()
    }
  }, [eventId, isAuthenticated, navigate])

  useEffect(() => {
    if (accessCheck?.hasAccess && eventId) {
      fetchParticipants()
    }
  }, [accessCheck, eventId, currentPage, filters])

  const checkAccess = async () => {
    if (!eventId) return

    try {
      setLoading(true)
      const response = await participantService.checkParticipantAccess(eventId)
      
      if (response.success) {
        setAccessCheck(response.data!)
      } else {
        setError(response.error || 'Failed to check access')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const fetchParticipants = async () => {
    if (!eventId || !accessCheck?.hasAccess) return

    try {
      setLoading(true)
      const response = await participantService.getEventParticipants(
        eventId,
        currentPage,
        pageSize,
        filters
      )
      
      if (response.success) {
        setParticipants(response.data!)
      } else {
        setError(response.error || 'Failed to fetch participants')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const handleFilterChange = (newFilters: Partial<ParticipantFilters>) => {
    setFilters(prev => ({ ...prev, ...newFilters }))
    setCurrentPage(1)
  }

  const clearFilters = () => {
    setFilters({})
    setCurrentPage(1)
  }

  const getMembershipTierBadge = (tier: string) => {
    const styles = {
      'Platinum': 'bg-gray-500/20 text-gray-300 border-gray-500/30',
      'Diamond': 'bg-blue-500/20 text-blue-400 border-blue-500/30',
      'Black Card': 'bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30'
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

  const getPrivacyLevelIndicator = (level: number) => {
    const colors = {
      1: 'bg-green-500/20 text-green-400',
      2: 'bg-blue-500/20 text-blue-400', 
      3: 'bg-yellow-500/20 text-yellow-400',
      4: 'bg-orange-500/20 text-orange-400',
      5: 'bg-red-500/20 text-red-400'
    }
    
    return (
      <div className={`w-2 h-2 rounded-full ${colors[level as keyof typeof colors]}`} 
           title={`隱私等級 ${level}`} />
    )
  }

  // Loading state
  if (loading && !participants) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
        <div className="luxury-glass p-8 rounded-2xl text-center">
          <div className="w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-luxury-platinum">載入參與者資訊中...</p>
        </div>
      </div>
    )
  }

  // No access state
  if (!accessCheck?.hasAccess) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="mb-6">
            <Link
              to={`/events/${eventId}`}
              className="inline-flex items-center gap-2 text-luxury-gold hover:text-luxury-gold/80 mb-4"
            >
              <ArrowLeft className="w-4 h-4" />
              返回活動詳情
            </Link>
          </div>

          <div className="luxury-glass p-8 rounded-2xl text-center">
            <Lock className="w-16 h-16 text-luxury-gold mx-auto mb-4" />
            <h2 className="text-2xl font-luxury font-bold text-luxury-gold mb-4">
              需要付費才能查看參與者
            </h2>
            <p className="text-luxury-platinum/80 mb-6">
              只有已付費參與此活動的會員才能查看其他參與者資訊
            </p>
            
            {accessCheck?.paymentStatus === 'pending' && (
              <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4 mb-6">
                <div className="flex items-center justify-center gap-2 text-yellow-400">
                  <CreditCard className="w-5 h-5" />
                  <p>您的報名付款正在處理中，完成付款後即可查看參與者</p>
                </div>
              </div>
            )}

            <div className="flex justify-center gap-4">
              <Link
                to={`/events/${eventId}/register`}
                className="luxury-button"
              >
                立即報名
              </Link>
              <Link
                to={`/events/${eventId}`}
                className="px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
              >
                查看活動詳情
              </Link>
            </div>
          </div>
        </div>
      </div>
    )
  }

  // Error state
  if (error) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="luxury-glass p-8 rounded-2xl text-center">
            <AlertCircle className="w-16 h-16 text-red-400 mx-auto mb-4" />
            <h2 className="text-2xl font-luxury font-bold text-luxury-gold mb-4">載入失敗</h2>
            <p className="text-luxury-platinum/80 mb-6">{error}</p>
            <button
              onClick={() => window.location.reload()}
              className="luxury-button"
            >
              重新載入
            </button>
          </div>
        </div>
      </div>
    )
  }

  const totalPages = participants ? Math.ceil(participants.totalCount / pageSize) : 1

  return (
    <div className="min-h-screen bg-luxury-midnight-black">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <Link
            to={`/events/${eventId}`}
            className="inline-flex items-center gap-2 text-luxury-gold hover:text-luxury-gold/80 mb-4"
          >
            <ArrowLeft className="w-4 h-4" />
            返回活動詳情
          </Link>
          
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-luxury font-bold text-luxury-gold mb-2">
                活動參與者
              </h1>
              <p className="text-luxury-platinum/80">
                與其他尊貴會員建立聯繫
              </p>
            </div>
            
            <div className="flex items-center gap-4">
              <button
                onClick={() => setShowFilters(!showFilters)}
                className="inline-flex items-center gap-2 px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
              >
                <Filter className="w-4 h-4" />
                篩選
              </button>
              
              <Link
                to={`/events/${eventId}/privacy-settings`}
                className="inline-flex items-center gap-2 px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors"
              >
                <Settings className="w-4 h-4" />
                隱私設定
              </Link>
            </div>
          </div>
        </div>

        {/* Statistics */}
        {participants && (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            <div className="luxury-glass p-6 rounded-2xl">
              <div className="flex items-center gap-3">
                <Users className="w-8 h-8 text-luxury-gold" />
                <div>
                  <p className="text-2xl font-bold text-luxury-gold">{participants.paidParticipantCount}</p>
                  <p className="text-luxury-platinum/80 text-sm">已付費參與者</p>
                </div>
              </div>
            </div>
            
            <div className="luxury-glass p-6 rounded-2xl">
              <div className="flex items-center gap-3">
                <Crown className="w-8 h-8 text-blue-400" />
                <div>
                  <p className="text-2xl font-bold text-luxury-gold">{participants.participantCountByTier['Diamond'] || 0}</p>
                  <p className="text-luxury-platinum/80 text-sm">Diamond 會員</p>
                </div>
              </div>
            </div>
            
            <div className="luxury-glass p-6 rounded-2xl">
              <div className="flex items-center gap-3">
                <Shield className="w-8 h-8 text-luxury-gold" />
                <div>
                  <p className="text-2xl font-bold text-luxury-gold">{participants.participantCountByTier['Black Card'] || 0}</p>
                  <p className="text-luxury-platinum/80 text-sm">Black Card 會員</p>
                </div>
              </div>
            </div>
            
            <div className="luxury-glass p-6 rounded-2xl">
              <div className="flex items-center gap-3">
                <Eye className="w-8 h-8 text-green-400" />
                <div>
                  <p className="text-2xl font-bold text-luxury-gold">{participants.participants.length}</p>
                  <p className="text-luxury-platinum/80 text-sm">可查看資料</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Filters */}
        {showFilters && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="luxury-glass p-6 rounded-2xl mb-8"
          >
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <div>
                <label className="block text-sm font-medium text-luxury-platinum/80 mb-2">
                  搜尋
                </label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-luxury-platinum/60" />
                  <input
                    type="text"
                    placeholder="搜尋參與者..."
                    value={filters.search || ''}
                    onChange={(e) => handleFilterChange({ search: e.target.value })}
                    className="w-full pl-10 pr-4 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/60 focus:border-luxury-gold focus:outline-none"
                  />
                </div>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-luxury-platinum/80 mb-2">
                  會員等級
                </label>
                <select
                  value={filters.membershipTier || ''}
                  onChange={(e) => handleFilterChange({ membershipTier: e.target.value || undefined })}
                  className="w-full px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum focus:border-luxury-gold focus:outline-none"
                >
                  <option value="">所有等級</option>
                  <option value="Platinum">Platinum</option>
                  <option value="Diamond">Diamond</option>
                  <option value="Black Card">Black Card</option>
                </select>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-luxury-platinum/80 mb-2">
                  職業領域
                </label>
                <input
                  type="text"
                  placeholder="如：科技、金融、醫療..."
                  value={filters.profession || ''}
                  onChange={(e) => handleFilterChange({ profession: e.target.value || undefined })}
                  className="w-full px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/60 focus:border-luxury-gold focus:outline-none"
                />
              </div>
              
              <div className="flex items-end">
                <button
                  onClick={clearFilters}
                  className="w-full px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
                >
                  清除篩選
                </button>
              </div>
            </div>
          </motion.div>
        )}

        {/* Participants Grid */}
        {participants && participants.participants.length > 0 ? (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 mb-8">
              {participants.participants.map((participant) => (
                <ParticipantCard 
                  key={participant.id} 
                  participant={participant} 
                  eventId={eventId!}
                  viewerAccess={participants.viewerAccess}
                />
              ))}
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex justify-center items-center gap-2">
                <button
                  onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
                  disabled={currentPage === 1}
                  className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  上一頁
                </button>
                
                <span className="px-4 py-2 text-luxury-platinum">
                  第 {currentPage} 頁，共 {totalPages} 頁
                </span>
                
                <button
                  onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                  disabled={currentPage === totalPages}
                  className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  下一頁
                </button>
              </div>
            )}
          </>
        ) : (
          <div className="luxury-glass p-12 rounded-2xl text-center">
            <Users className="w-16 h-16 text-luxury-platinum/60 mx-auto mb-4" />
            <h3 className="text-xl font-medium text-luxury-platinum mb-2">
              目前沒有符合條件的參與者
            </h3>
            <p className="text-luxury-platinum/60">
              請調整篩選條件或稍後再試
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

// Participant Card Component
const ParticipantCard: React.FC<{
  participant: FilteredParticipantInfo
  eventId: string
  viewerAccess: any
}> = ({ participant, eventId, viewerAccess }) => {
  const [showContactModal, setShowContactModal] = useState(false)

  const getMembershipTierBadge = (tier: string) => {
    const styles = {
      'Platinum': 'bg-gray-500/20 text-gray-300 border-gray-500/30',
      'Diamond': 'bg-blue-500/20 text-blue-400 border-blue-500/30',
      'Black Card': 'bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30'
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

  const getPrivacyLevelIndicator = (level: number) => {
    const colors = {
      1: 'bg-green-500/20 text-green-400',
      2: 'bg-blue-500/20 text-blue-400', 
      3: 'bg-yellow-500/20 text-yellow-400',
      4: 'bg-orange-500/20 text-orange-400',
      5: 'bg-red-500/20 text-red-400'
    }
    
    return (
      <div className={`w-2 h-2 rounded-full ${colors[level as keyof typeof colors]}`} 
           title={`隱私等級 ${level}`} />
    )
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="luxury-glass p-6 rounded-2xl hover:bg-luxury-midnight-black/60 transition-colors"
    >
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-12 h-12 bg-luxury-gold/20 rounded-full flex items-center justify-center">
            <span className="text-luxury-gold font-bold text-lg">
              {participant.displayName.charAt(0)}
            </span>
          </div>
          <div>
            <h3 className="font-medium text-luxury-platinum">
              {participant.displayName}
            </h3>
            {participant.ageRange && (
              <p className="text-sm text-luxury-platinum/60">
                {participant.ageRange}
              </p>
            )}
          </div>
        </div>
        {getPrivacyLevelIndicator(participant.privacyLevel)}
      </div>

      <div className="space-y-2 mb-4">
        {participant.profession && (
          <p className="text-sm text-luxury-platinum/80">
            {participant.profession}
          </p>
        )}
        
        {participant.company && (
          <p className="text-sm text-luxury-platinum/60">
            {participant.company}
          </p>
        )}
        
        {participant.city && (
          <p className="text-sm text-luxury-platinum/60">
            📍 {participant.city}
          </p>
        )}
      </div>

      <div className="flex items-center justify-between">
        {getMembershipTierBadge(participant.membershipTier)}
        
        {participant.canContact && viewerAccess.canInitiateContact && (
          <button
            onClick={() => setShowContactModal(true)}
            className="p-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors"
            title="聯繫此會員"
          >
            <MessageCircle className="w-4 h-4" />
          </button>
        )}
      </div>

      {participant.interests && participant.interests.length > 0 && (
        <div className="mt-4 pt-4 border-t border-luxury-gold/20">
          <div className="flex flex-wrap gap-1">
            {participant.interests.slice(0, 3).map((interest, index) => (
              <span
                key={index}
                className="px-2 py-1 bg-luxury-gold/10 text-luxury-gold text-xs rounded-full"
              >
                {interest}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Contact Modal */}
      {showContactModal && (
        <ContactModal
          participant={participant}
          eventId={eventId}
          onClose={() => setShowContactModal(false)}
        />
      )}
    </motion.div>
  )
}

// Contact Modal Component
const ContactModal: React.FC<{
  participant: FilteredParticipantInfo
  eventId: string
  onClose: () => void
}> = ({ participant, eventId, onClose }) => {
  const [message, setMessage] = useState('')
  const [sending, setSending] = useState(false)
  const [sent, setSent] = useState(false)

  const handleSendMessage = async () => {
    if (!message.trim()) return

    setSending(true)
    try {
      const response = await participantService.initiateContact(
        eventId,
        participant.id,
        message
      )
      
      if (response.success) {
        setSent(true)
        setTimeout(() => {
          onClose()
        }, 2000)
      }
    } catch (error) {
      console.error('Failed to send message:', error)
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-luxury-midnight-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="luxury-glass p-6 rounded-2xl max-w-md w-full"
      >
        {sent ? (
          <div className="text-center">
            <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
              <MessageCircle className="w-8 h-8 text-green-400" />
            </div>
            <h3 className="text-xl font-bold text-luxury-gold mb-2">
              訊息已發送
            </h3>
            <p className="text-luxury-platinum/80">
              您的聯繫請求已送達 {participant.displayName}
            </p>
          </div>
        ) : (
          <>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-bold text-luxury-gold">
                聯繫 {participant.displayName}
              </h3>
              <button
                onClick={onClose}
                className="p-2 text-luxury-platinum/60 hover:text-luxury-platinum"
              >
                ✕
              </button>
            </div>

            <div className="mb-4">
              <label className="block text-sm font-medium text-luxury-platinum/80 mb-2">
                訊息內容
              </label>
              <textarea
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                rows={4}
                className="w-full px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/60 focus:border-luxury-gold focus:outline-none resize-none"
                placeholder="請輸入您想要傳達的訊息..."
              />
            </div>

            <div className="flex justify-end gap-3">
              <button
                onClick={onClose}
                className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleSendMessage}
                disabled={!message.trim() || sending}
                className="px-4 py-2 bg-luxury-gold text-luxury-midnight-black rounded-lg hover:bg-luxury-gold/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
              >
                {sending ? '發送中...' : '發送訊息'}
              </button>
            </div>
          </>
        )}
      </motion.div>
    </div>
  )
}

export default EventParticipants