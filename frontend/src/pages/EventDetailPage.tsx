import { useState, useEffect } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { motion } from 'framer-motion'
import { 
  Calendar, MapPin, Users, Star, Crown, Clock, Shield, 
  ChevronLeft, ChevronRight, Heart, Share2,
  Shirt, Check, AlertCircle
} from 'lucide-react'
import { eventService } from '../services/eventService'

const EventDetailPage = () => {
  const { id } = useParams()
  const navigate = useNavigate()
  const { user, isAuthenticated } = useAuth()
  const [event, setEvent] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [currentImageIndex, setCurrentImageIndex] = useState(0)
  const [showRegistrationModal, setShowRegistrationModal] = useState(false)
  const [isRegistering, setIsRegistering] = useState(false)

  useEffect(() => {
    const fetchEvent = async () => {
      setLoading(true)
      
      try {
        if (!id) {
          throw new Error('Event ID is required')
        }
        
        const response = await eventService.getEvent(id)
        
        if (response.success && response.data) {
          setEvent(response.data)
        } else {
          console.error('Failed to fetch event:', response.error)
        }
      } catch (error) {
        console.error('Error fetching event:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchEvent()
  }, [id])

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

  const getDressCodeText = (level: number) => {
    const codes = {
      1: '休閒',
      2: '商務休閒',
      3: '正式',
      4: '晚宴正裝',
      5: '黑領結/長禮服'
    }
    return codes[level as keyof typeof codes] || '未指定'
  }

  const getExclusivityColor = (level: string) => {
    switch (level) {
      case 'VIP':
        return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
      case 'VVIP':
        return 'bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30'
      case 'Invitation Only':
        return 'bg-purple-500/20 text-purple-400 border-purple-500/30'
      default:
        return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
    }
  }

  const handleRegister = async () => {
    setIsRegistering(true)
    
    try {
      if (!id) {
        throw new Error('Event ID is required')
      }
      
      const response = await eventService.registerForEvent(id)
      
      if (response.success) {
        setShowRegistrationModal(false)
        // Show success message
        alert('報名成功！您已成功報名此活動。')
      } else {
        throw new Error(response.error || 'Registration failed')
      }
    } catch (error) {
      console.error('Registration failed:', error)
      alert('報名失敗，請稍後再試。')
    } finally {
      setIsRegistering(false)
    }
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
        <div className="luxury-glass p-8 rounded-2xl text-center">
          <div className="w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-luxury-platinum">載入活動資訊中...</p>
        </div>
      </div>
    )
  }

  if (!event) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
        <div className="luxury-glass p-8 rounded-2xl text-center">
          <AlertCircle className="h-12 w-12 text-red-400 mx-auto mb-4" />
          <h2 className="text-xl font-luxury text-luxury-gold mb-2">活動未找到</h2>
          <p className="text-luxury-platinum/80 mb-4">抱歉，您要查看的活動不存在或已被移除。</p>
          <Link to="/events" className="luxury-button">
            返回活動列表
          </Link>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black">
      {/* Hero Section with Image Gallery */}
      <div className="relative h-[60vh] overflow-hidden">
        <div className="relative h-full">
          <img
            src={event.images[currentImageIndex]}
            alt={event.name}
            className="w-full h-full object-cover"
          />
          <div className="absolute inset-0 bg-luxury-midnight-black/50"></div>
          
          {/* Image Navigation */}
          {event.images.length > 1 && (
            <>
              <button
                onClick={() => setCurrentImageIndex(prev => 
                  prev === 0 ? event.images.length - 1 : prev - 1
                )}
                className="absolute left-4 top-1/2 transform -translate-y-1/2 w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors"
              >
                <ChevronLeft className="h-6 w-6" />
              </button>
              <button
                onClick={() => setCurrentImageIndex(prev => 
                  prev === event.images.length - 1 ? 0 : prev + 1
                )}
                className="absolute right-4 top-1/2 transform -translate-y-1/2 w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors"
              >
                <ChevronRight className="h-6 w-6" />
              </button>
              
              {/* Image Indicators */}
              <div className="absolute bottom-4 left-1/2 transform -translate-x-1/2 flex space-x-2">
                {event.images.map((_: any, index: number) => (
                  <button
                    key={index}
                    onClick={() => setCurrentImageIndex(index)}
                    className={`w-3 h-3 rounded-full transition-colors ${
                      index === currentImageIndex ? 'bg-luxury-gold' : 'bg-white/30'
                    }`}
                  />
                ))}
              </div>
            </>
          )}
        </div>
        
        {/* Back Button */}
        <div className="absolute top-8 left-8">
          <Link
            to="/events"
            className="inline-flex items-center px-4 py-2 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-lg text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors"
          >
            <ChevronLeft className="h-5 w-5 mr-1" />
            返回
          </Link>
        </div>

        {/* Action Buttons */}
        <div className="absolute top-8 right-8 flex space-x-2">
          <button className="w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors">
            <Heart className="h-5 w-5" />
          </button>
          <button className="w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors">
            <Share2 className="h-5 w-5" />
          </button>
        </div>
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2">
            <motion.div
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8 }}
              className="space-y-8"
            >
              {/* Event Header */}
              <div>
                <div className="flex items-start justify-between mb-4">
                  <div>
                    <span className={`inline-block px-3 py-1 rounded-full text-xs font-medium border ${getExclusivityColor(event.exclusivityLevel)} mb-2`}>
                      {event.exclusivityLevel}
                    </span>
                    <h1 className="text-4xl font-luxury font-bold text-luxury-gold mb-2">
                      {event.name}
                    </h1>
                    <p className="text-luxury-platinum/80 text-lg">
                      主辦：{event.organizer}
                    </p>
                  </div>
                </div>

                <div className="flex flex-wrap gap-2">
                  {event.tags.map((tag: string, index: number) => (
                    <span
                      key={index}
                      className="px-3 py-1 bg-luxury-gold/20 text-luxury-gold text-sm rounded-full"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>

              {/* Event Details */}
              <div className="luxury-glass p-6 rounded-2xl">
                <h2 className="text-2xl font-luxury font-semibold text-luxury-gold mb-4">
                  活動詳情
                </h2>
                <div className="prose prose-invert max-w-none">
                  <p className="text-luxury-platinum/80 leading-relaxed whitespace-pre-line">
                    {event.description}
                  </p>
                </div>
              </div>

              {/* Key Information */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="luxury-glass p-6 rounded-2xl">
                  <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                    時間地點
                  </h3>
                  <div className="space-y-3">
                    <div className="flex items-start">
                      <Calendar className="h-5 w-5 text-luxury-gold mr-3 mt-0.5" />
                      <div>
                        <p className="text-luxury-platinum font-medium">
                          {formatDate(event.dateTime)}
                        </p>
                        <p className="text-luxury-platinum/80 text-sm">
                          {formatTime(event.dateTime)}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-start">
                      <MapPin className="h-5 w-5 text-luxury-gold mr-3 mt-0.5" />
                      <div>
                        <p className="text-luxury-platinum font-medium">
                          {event.venue.name}
                        </p>
                        <p className="text-luxury-platinum/80 text-sm">
                          {event.venue.address}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center">
                      <Clock className="h-5 w-5 text-luxury-gold mr-3" />
                      <div>
                        <p className="text-luxury-platinum font-medium">
                          報名截止
                        </p>
                        <p className="text-luxury-platinum/80 text-sm">
                          {formatDate(event.registrationDeadline)}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="luxury-glass p-6 rounded-2xl">
                  <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                    活動規格
                  </h3>
                  <div className="space-y-3">
                    <div className="flex items-center">
                      <Users className="h-5 w-5 text-luxury-gold mr-3" />
                      <div>
                        <p className="text-luxury-platinum font-medium">
                          參與人數
                        </p>
                        <p className="text-luxury-platinum/80 text-sm">
                          {event.currentAttendees}/{event.capacity} 人
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center">
                      <Shirt className="h-5 w-5 text-luxury-gold mr-3" />
                      <div>
                        <p className="text-luxury-platinum font-medium">
                          服裝規範
                        </p>
                        <p className="text-luxury-platinum/80 text-sm">
                          {getDressCodeText(event.dressCode)}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center">
                      <Crown className="h-5 w-5 text-luxury-gold mr-3" />
                      <div>
                        <p className="text-luxury-platinum font-medium">
                          獨家等級
                        </p>
                        <p className="text-luxury-platinum/80 text-sm">
                          {event.exclusivityLevel}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Amenities & Privacy */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="luxury-glass p-6 rounded-2xl">
                  <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                    專屬服務
                  </h3>
                  <ul className="space-y-2">
                    {event.amenities.map((amenity: string, index: number) => (
                      <li key={index} className="flex items-center text-luxury-platinum/80">
                        <Check className="h-4 w-4 text-luxury-gold mr-2" />
                        {amenity}
                      </li>
                    ))}
                  </ul>
                </div>

                <div className="luxury-glass p-6 rounded-2xl">
                  <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                    隱私保障
                  </h3>
                  <ul className="space-y-2">
                    {event.privacyGuarantees.map((guarantee: string, index: number) => (
                      <li key={index} className="flex items-center text-luxury-platinum/80">
                        <Shield className="h-4 w-4 text-luxury-gold mr-2" />
                        {guarantee}
                      </li>
                    ))}
                  </ul>
                </div>
              </div>

              {/* Requirements */}
              <div className="luxury-glass p-6 rounded-2xl">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                  參與要求
                </h3>
                <div className="space-y-2">
                  {event.requirements.map((req: any, index: number) => (
                    <div key={index} className="flex items-center text-luxury-platinum/80">
                      <AlertCircle className="h-4 w-4 text-luxury-gold mr-2" />
                      {req.description}
                    </div>
                  ))}
                </div>
              </div>
            </motion.div>
          </div>

          {/* Sidebar */}
          <div className="lg:col-span-1">
            <motion.div
              initial={{ opacity: 0, x: 30 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.8, delay: 0.2 }}
              className="sticky top-24"
            >
              <div className="luxury-glass p-6 rounded-2xl">
                <div className="text-center mb-6">
                  <div className="text-3xl font-bold text-luxury-gold mb-2">
                    {event.pricing.vvip ? `NT$ ${event.pricing.vvip.toLocaleString()}` : 
                     event.pricing.vip ? `NT$ ${event.pricing.vip.toLocaleString()}` : '價格洽詢'}
                  </div>
                  <p className="text-luxury-platinum/80 text-sm">
                    {event.pricing.vvip ? 'VVIP 價格' : 'VIP 價格'}
                  </p>
                </div>

                <div className="space-y-4 mb-6">
                  <div className="flex justify-between items-center">
                    <span className="text-luxury-platinum/80">報名人數</span>
                    <span className="text-luxury-gold font-medium">
                      {event.currentAttendees}/{event.capacity}
                    </span>
                  </div>
                  <div className="w-full bg-luxury-midnight-black/50 rounded-full h-2">
                    <div 
                      className="bg-luxury-gold h-2 rounded-full transition-all duration-300"
                      style={{ width: `${(event.currentAttendees / event.capacity) * 100}%` }}
                    ></div>
                  </div>
                  <div className="flex justify-between items-center text-sm">
                    <span className="text-luxury-platinum/80">剩餘名額</span>
                    <span className="text-luxury-gold">
                      {event.capacity - event.currentAttendees} 個
                    </span>
                  </div>
                </div>

                {isAuthenticated ? (
                  <div className="space-y-3 mb-4">
                    <button
                      onClick={() => navigate(`/events/${id}/register`)}
                      className="w-full luxury-button py-3"
                      disabled={event.currentAttendees >= event.capacity}
                    >
                      {event.currentAttendees >= event.capacity ? '已額滿' : '立即報名'}
                    </button>
                    
                    <button
                      onClick={() => navigate(`/events/${id}/participants`)}
                      className="w-full px-4 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors flex items-center justify-center gap-2"
                    >
                      <Users className="w-4 h-4" />
                      查看參與者
                    </button>
                  </div>
                ) : (
                  <div className="space-y-2 mb-4">
                    <button
                      onClick={() => navigate('/login')}
                      className="w-full luxury-button py-3"
                    >
                      登入後報名
                    </button>
                    <p className="text-center text-luxury-platinum/60 text-xs">
                      需要登入會員帳號才能報名活動
                    </p>
                  </div>
                )}

                <div className="text-center">
                  <p className="text-luxury-platinum/60 text-xs">
                    * 所有報名需經過審核
                  </p>
                  <p className="text-luxury-platinum/60 text-xs">
                    * 48小時內無條件退款
                  </p>
                </div>
              </div>

              {/* Venue Info */}
              <div className="luxury-glass p-6 rounded-2xl mt-6">
                <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                  場地資訊
                </h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-luxury-platinum font-medium">{event.venue.name}</span>
                    <div className="flex items-center">
                      {Array.from({ length: event.venue.rating }).map((_, i) => (
                        <Star key={i} className="h-4 w-4 text-luxury-gold fill-current" />
                      ))}
                    </div>
                  </div>
                  <p className="text-luxury-platinum/80 text-sm">{event.venue.address}</p>
                  <div className="space-y-1">
                    {event.venue.amenities.map((amenity: string, index: number) => (
                      <div key={index} className="flex items-center text-luxury-platinum/80 text-sm">
                        <Check className="h-3 w-3 text-luxury-gold mr-2" />
                        {amenity}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          </div>
        </div>
      </div>

      {/* Registration Modal */}
      {showRegistrationModal && (
        <div className="fixed inset-0 bg-luxury-midnight-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="luxury-glass p-8 rounded-2xl max-w-md w-full"
          >
            <h3 className="text-2xl font-luxury font-bold text-luxury-gold mb-4">
              確認報名
            </h3>
            <p className="text-luxury-platinum/80 mb-6">
              您確定要報名參加「{event.name}」嗎？
            </p>
            
            <div className="space-y-4 mb-6">
              <div className="flex justify-between">
                <span className="text-luxury-platinum/80">活動費用</span>
                <span className="text-luxury-gold font-medium">
                  NT$ {(event.pricing.vvip || event.pricing.vip).toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-luxury-platinum/80">服務費</span>
                <span className="text-luxury-platinum/80">包含在內</span>
              </div>
              <hr className="border-luxury-gold/20" />
              <div className="flex justify-between text-lg font-medium">
                <span className="text-luxury-platinum">總計</span>
                <span className="text-luxury-gold">
                  NT$ {(event.pricing.vvip || event.pricing.vip).toLocaleString()}
                </span>
              </div>
            </div>

            <div className="flex space-x-4">
              <button
                onClick={() => setShowRegistrationModal(false)}
                className="flex-1 px-4 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleRegister}
                disabled={isRegistering}
                className="flex-1 luxury-button py-3 disabled:opacity-50"
              >
                {isRegistering ? '處理中...' : '確認報名'}
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </div>
  )
}

export default EventDetailPage