import React, { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { motion } from 'framer-motion'
import { 
  ArrowLeft, 
  Shield, 
  Eye, 
  EyeOff,
  MessageCircle, 
  Phone,
  Mail,
  Users,
  Settings,
  Save,
  Info,
  AlertCircle
} from 'lucide-react'
import participantService from '../services/participantService'

interface PrivacySettings {
  privacy_level: number
  allow_contact: boolean
  show_in_list: boolean
}

const EventPrivacySettings: React.FC = () => {
  const { eventId } = useParams<{ eventId: string }>()
  const navigate = useNavigate()
  const { user, isAuthenticated } = useAuth()

  // State management
  const [settings, setSettings] = useState<PrivacySettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  useEffect(() => {
    if (!isAuthenticated) {
      navigate('/login')
      return
    }

    if (eventId) {
      fetchSettings()
    }
  }, [eventId, isAuthenticated, navigate])

  const fetchSettings = async () => {
    if (!eventId) return

    try {
      setLoading(true)
      const response = await participantService.getPrivacySettings(eventId)
      
      if (response.success) {
        setSettings(response.data)
      } else {
        setError(response.error || 'Failed to fetch privacy settings')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const saveSettings = async () => {
    if (!eventId || !settings) return

    try {
      setSaving(true)
      setError(null)
      
      const response = await participantService.updatePrivacySettings(eventId, {
        privacyLevel: settings.privacy_level,
        allowContact: settings.allow_contact,
        showInList: settings.show_in_list
      })
      
      if (response.success) {
        setSuccess('隱私設定已成功更新')
        setTimeout(() => setSuccess(null), 3000)
      } else {
        setError(response.error || 'Failed to update privacy settings')
      }
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setSaving(false)
    }
  }

  const getPrivacyLevelDescription = (level: number) => {
    const descriptions = {
      1: {
        title: '公開資料',
        description: '顯示基本資訊：姓名縮寫、年齡範圍、職業類別、會員等級',
        visibility: '所有付費參與者可見'
      },
      2: {
        title: '半私人資料',
        description: '顯示完整名字、公司行業、經驗範圍、城市',
        visibility: '所有付費參與者可見'
      },
      3: {
        title: '選擇性分享',
        description: '顯示全名、公司名稱、具體興趣、專業成就',
        visibility: '所有付費參與者可見'
      },
      4: {
        title: '增強資料',
        description: '顯示聯絡資訊、社交連結、詳細履歷',
        visibility: 'Diamond 和 Black Card 會員可見'
      },
      5: {
        title: '完全公開',
        description: '顯示直接聯絡方式、個人興趣、網絡連接',
        visibility: 'Diamond 和 Black Card 會員可見'
      }
    }
    return descriptions[level as keyof typeof descriptions]
  }

  const getPrivacyLevelColor = (level: number) => {
    const colors = {
      1: 'border-green-500/30 bg-green-500/10 text-green-400',
      2: 'border-blue-500/30 bg-blue-500/10 text-blue-400',
      3: 'border-yellow-500/30 bg-yellow-500/10 text-yellow-400',
      4: 'border-orange-500/30 bg-orange-500/10 text-orange-400',
      5: 'border-red-500/30 bg-red-500/10 text-red-400'
    }
    return colors[level as keyof typeof colors]
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
        <div className="luxury-glass p-8 rounded-2xl text-center">
          <div className="w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-luxury-platinum">載入隱私設定中...</p>
        </div>
      </div>
    )
  }

  if (!settings) {
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

  return (
    <div className="min-h-screen bg-luxury-midnight-black">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center gap-4 mb-4">
            <Link
              to={`/events/${eventId}/participants`}
              className="inline-flex items-center gap-2 text-luxury-gold hover:text-luxury-gold/80"
            >
              <ArrowLeft className="w-4 h-4" />
              返回參與者列表
            </Link>
            
            <Link
              to={`/events/${eventId}`}
              className="text-luxury-platinum/60 hover:text-luxury-platinum"
            >
              活動詳情
            </Link>
          </div>
          
          <div className="flex items-center gap-3 mb-2">
            <Shield className="w-8 h-8 text-luxury-gold" />
            <h1 className="text-3xl font-luxury font-bold text-luxury-gold">
              隱私設定
            </h1>
          </div>
          <p className="text-luxury-platinum/80">
            控制您在此活動中的資訊顯示程度和聯繫偏好
          </p>
        </div>

        {/* Success Message */}
        {success && (
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mb-6 bg-green-500/10 border border-green-500/20 rounded-lg p-4"
          >
            <div className="flex items-center gap-2 text-green-400">
              <Save className="w-5 h-5" />
              <p>{success}</p>
            </div>
          </motion.div>
        )}

        {/* Error Message */}
        {error && (
          <div className="mb-6 bg-red-500/10 border border-red-500/20 rounded-lg p-4">
            <div className="flex items-center gap-2 text-red-400">
              <AlertCircle className="w-5 h-5" />
              <p>{error}</p>
            </div>
          </div>
        )}

        <div className="space-y-8">
          {/* Privacy Level Settings */}
          <div className="luxury-glass p-6 rounded-2xl">
            <div className="flex items-center gap-3 mb-6">
              <Eye className="w-6 h-6 text-luxury-gold" />
              <h2 className="text-xl font-luxury font-semibold text-luxury-gold">
                資訊公開程度
              </h2>
            </div>

            <div className="space-y-4">
              {[1, 2, 3, 4, 5].map((level) => {
                const description = getPrivacyLevelDescription(level)
                const isSelected = settings.privacy_level === level
                
                return (
                  <motion.div
                    key={level}
                    className={`p-4 rounded-xl border-2 cursor-pointer transition-all ${
                      isSelected 
                        ? getPrivacyLevelColor(level)
                        : 'border-luxury-gold/20 bg-luxury-midnight-black/30 hover:border-luxury-gold/30'
                    }`}
                    onClick={() => setSettings({ ...settings, privacy_level: level })}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <div className={`w-3 h-3 rounded-full ${getPrivacyLevelColor(level).split(' ')[2]}`} />
                          <h3 className="font-medium text-luxury-platinum">
                            等級 {level} - {description.title}
                          </h3>
                        </div>
                        <p className="text-sm text-luxury-platinum/80 mb-2">
                          {description.description}
                        </p>
                        <p className="text-xs text-luxury-platinum/60">
                          可見範圍：{description.visibility}
                        </p>
                      </div>
                      
                      {isSelected && (
                        <div className="ml-4">
                          <div className="w-5 h-5 bg-luxury-gold rounded-full flex items-center justify-center">
                            <div className="w-2 h-2 bg-luxury-midnight-black rounded-full" />
                          </div>
                        </div>
                      )}
                    </div>
                  </motion.div>
                )
              })}
            </div>
          </div>

          {/* Contact Preferences */}
          <div className="luxury-glass p-6 rounded-2xl">
            <div className="flex items-center gap-3 mb-6">
              <MessageCircle className="w-6 h-6 text-luxury-gold" />
              <h2 className="text-xl font-luxury font-semibold text-luxury-gold">
                聯繫偏好
              </h2>
            </div>

            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-xl">
                <div className="flex items-center gap-3">
                  <MessageCircle className="w-5 h-5 text-luxury-platinum/60" />
                  <div>
                    <h3 className="font-medium text-luxury-platinum">
                      允許其他會員聯繫我
                    </h3>
                    <p className="text-sm text-luxury-platinum/60">
                      其他付費參與者可以向您發送聯繫請求
                    </p>
                  </div>
                </div>
                
                <button
                  onClick={() => setSettings({ ...settings, allow_contact: !settings.allow_contact })}
                  className={`relative w-12 h-6 rounded-full transition-colors ${
                    settings.allow_contact ? 'bg-luxury-gold' : 'bg-luxury-platinum/20'
                  }`}
                >
                  <div className={`absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    settings.allow_contact ? 'translate-x-6' : 'translate-x-0.5'
                  }`} />
                </button>
              </div>

              <div className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-xl">
                <div className="flex items-center gap-3">
                  <Users className="w-5 h-5 text-luxury-platinum/60" />
                  <div>
                    <h3 className="font-medium text-luxury-platinum">
                      顯示在參與者列表中
                    </h3>
                    <p className="text-sm text-luxury-platinum/60">
                      在活動參與者列表中顯示您的資訊
                    </p>
                  </div>
                </div>
                
                <button
                  onClick={() => setSettings({ ...settings, show_in_list: !settings.show_in_list })}
                  className={`relative w-12 h-6 rounded-full transition-colors ${
                    settings.show_in_list ? 'bg-luxury-gold' : 'bg-luxury-platinum/20'
                  }`}
                >
                  <div className={`absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
                    settings.show_in_list ? 'translate-x-6' : 'translate-x-0.5'
                  }`} />
                </button>
              </div>
            </div>
          </div>

          {/* Information Notice */}
          <div className="luxury-glass p-6 rounded-2xl border border-blue-500/20">
            <div className="flex items-start gap-3">
              <Info className="w-6 h-6 text-blue-400 mt-0.5" />
              <div>
                <h3 className="font-medium text-blue-400 mb-2">
                  隱私保護說明
                </h3>
                <div className="space-y-2 text-sm text-luxury-platinum/80">
                  <p>• 只有已付費參與此活動的會員才能查看參與者資訊</p>
                  <p>• 您可以隨時調整隱私等級，變更將立即生效</p>
                  <p>• 聯繫請求會通過平台系統發送，不會直接暴露個人聯絡方式</p>
                  <p>• 所有參與者查看記錄都會被記錄，確保安全性</p>
                  <p>• Diamond 和 Black Card 會員享有更高等級的資訊查看權限</p>
                </div>
              </div>
            </div>
          </div>

          {/* Save Button */}
          <div className="flex justify-end gap-4">
            <Link
              to={`/events/${eventId}/participants`}
              className="px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
            >
              取消
            </Link>
            
            <button
              onClick={saveSettings}
              disabled={saving}
              className="luxury-button px-8 py-3 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              {saving ? (
                <>
                  <div className="w-4 h-4 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin" />
                  儲存中...
                </>
              ) : (
                <>
                  <Save className="w-4 h-4" />
                  儲存設定
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default EventPrivacySettings