import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  Mail, Briefcase, 
  Crown, Star, Calendar, Edit3, Save, X, Plus,
  Award, TrendingUp, Users
} from 'lucide-react'
import { useAuth } from '../hooks/useAuth'

const ProfilePage = () => {
  const { updateProfile } = useAuth()
  const [isEditing, setIsEditing] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [profileData, setProfileData] = useState({
    firstName: '志明',
    lastName: '陳',
    email: 'chen.executive@example.com',
    phone: '+886-912-345-678',
    age: 52,
    profession: '科技公司CEO',
    annualIncome: 25000000,
    netWorth: 180000000,
    membershipTier: 'Diamond',
    privacyLevel: 3,
    bio: '專注於科技創新與社會責任的企業家，熱愛藝術收藏與高爾夫運動。',
    interests: ['科技創新', '藝術收藏', '高爾夫', '紅酒品鑑', '慈善事業'],
    profilePicture: '/api/placeholder/150/150'
  })

  const [editData, setEditData] = useState(profileData)
  const [newInterest, setNewInterest] = useState('')

  const membershipBenefits = {
    Platinum: ['參與精選社交活動', '基本身份驗證', '標準客服支援'],
    Diamond: ['VIP活動優先預訂', '專屬社交顧問', '私人活動邀請', '高端場地折扣'],
    'Black Card': ['獨家VVIP活動', '24/7禮賓服務', '客製化活動規劃', '全球合作夥伴特權']
  }

  const activityStats = {
    eventsAttended: 15,
    upcomingEvents: 3,
    totalSpent: 450000,
    memberSince: '2023-06-15'
  }

  const upcomingEvents = [
    {
      id: '1',
      name: '星空下的法式晚宴',
      date: '2024-12-27',
      status: 'confirmed'
    },
    {
      id: '2',
      name: '私人遊艇品酒之夜',
      date: '2024-12-15',
      status: 'pending'
    },
    {
      id: '3',
      name: '當代藝術收藏家沙龍',
      date: '2024-12-08',
      status: 'confirmed'
    }
  ]

  const handleEdit = () => {
    setIsEditing(true)
    setEditData(profileData)
  }

  const handleSave = async () => {
    setIsSaving(true)
    
    try {
      const result = await updateProfile(editData)
      
      if (result.success) {
        setProfileData(editData)
        setIsEditing(false)
      } else {
        console.error('Profile update failed:', result.error)
      }
    } catch (error) {
      console.error('Profile update error:', error)
    } finally {
      setIsSaving(false)
    }
  }

  const handleCancel = () => {
    setEditData(profileData)
    setIsEditing(false)
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    setEditData(prev => ({
      ...prev,
      [e.target.name]: e.target.value
    }))
  }

  const addInterest = () => {
    if (newInterest.trim() && !editData.interests.includes(newInterest.trim()) && editData.interests.length < 10) {
      setEditData(prev => ({
        ...prev,
        interests: [...prev.interests, newInterest.trim()]
      }))
      setNewInterest('')
    }
  }

  const removeInterest = (interest: string) => {
    setEditData(prev => ({
      ...prev,
      interests: prev.interests.filter(i => i !== interest)
    }))
  }

  const getMembershipColor = (tier: string) => {
    switch (tier) {
      case 'Platinum':
        return 'text-gray-400'
      case 'Diamond':
        return 'text-blue-400'
      case 'Black Card':
        return 'text-luxury-gold'
      default:
        return 'text-luxury-platinum'
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'confirmed':
        return 'bg-green-500/20 text-green-400 border-green-500/30'
      case 'pending':
        return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
      default:
        return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
    }
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Profile Section */}
          <div className="lg:col-span-1">
            <motion.div
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8 }}
              className="luxury-glass p-6 rounded-2xl"
            >
              <div className="text-center">
                <div className="relative inline-block mb-4">
                  <img
                    src={profileData.profilePicture}
                    alt="Profile"
                    className="w-24 h-24 rounded-full border-4 border-luxury-gold"
                  />
                  <div className="absolute -bottom-1 -right-1 w-8 h-8 bg-luxury-gold rounded-full flex items-center justify-center">
                    <Crown className="h-4 w-4 text-luxury-midnight-black" />
                  </div>
                </div>

                <h1 className="text-2xl font-luxury font-bold text-luxury-gold mb-2">
                  {profileData.firstName} {profileData.lastName}
                </h1>

                <div className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium mb-4 ${getMembershipColor(profileData.membershipTier)}`}>
                  <Crown className="h-4 w-4 mr-1" />
                  {profileData.membershipTier} 會員
                </div>

                <p className="text-luxury-platinum/80 text-sm mb-6">
                  {profileData.bio}
                </p>

                <div className="space-y-3 text-sm">
                  <div className="flex items-center justify-center text-luxury-platinum/80">
                    <Mail className="h-4 w-4 mr-2" />
                    {profileData.email}
                  </div>
                  <div className="flex items-center justify-center text-luxury-platinum/80">
                    <Briefcase className="h-4 w-4 mr-2" />
                    {profileData.profession}
                  </div>
                  <div className="flex items-center justify-center text-luxury-platinum/80">
                    <Calendar className="h-4 w-4 mr-2" />
                    會員自 {new Date(activityStats.memberSince).getFullYear()} 年
                  </div>
                </div>

                <button
                  onClick={handleEdit}
                  className="mt-6 luxury-button-outline w-full"
                >
                  <Edit3 className="h-4 w-4 mr-2" />
                  編輯個人資料
                </button>
              </div>
            </motion.div>

            {/* Membership Benefits */}
            <motion.div
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: 0.2 }}
              className="luxury-glass p-6 rounded-2xl mt-6"
            >
              <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
                會員權益
              </h3>
              <ul className="space-y-2">
                {membershipBenefits[profileData.membershipTier as keyof typeof membershipBenefits]?.map((benefit, index) => (
                  <li key={index} className="flex items-start text-luxury-platinum/80 text-sm">
                    <Star className="h-4 w-4 text-luxury-gold mr-2 mt-0.5 flex-shrink-0" />
                    {benefit}
                  </li>
                ))}
              </ul>
            </motion.div>
          </div>

          {/* Main Content */}
          <div className="lg:col-span-2">
            {/* Activity Stats */}
            <motion.div
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: 0.1 }}
              className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8"
            >
              <div className="luxury-glass p-4 rounded-xl text-center">
                <Users className="h-8 w-8 text-luxury-gold mx-auto mb-2" />
                <div className="text-2xl font-bold text-luxury-gold">{activityStats.eventsAttended}</div>
                <div className="text-luxury-platinum/80 text-sm">參與活動</div>
              </div>
              <div className="luxury-glass p-4 rounded-xl text-center">
                <Calendar className="h-8 w-8 text-luxury-gold mx-auto mb-2" />
                <div className="text-2xl font-bold text-luxury-gold">{activityStats.upcomingEvents}</div>
                <div className="text-luxury-platinum/80 text-sm">即將參與</div>
              </div>
              <div className="luxury-glass p-4 rounded-xl text-center">
                <TrendingUp className="h-8 w-8 text-luxury-gold mx-auto mb-2" />
                <div className="text-2xl font-bold text-luxury-gold">
                  NT$ {(activityStats.totalSpent / 1000).toFixed(0)}K
                </div>
                <div className="text-luxury-platinum/80 text-sm">累計消費</div>
              </div>
              <div className="luxury-glass p-4 rounded-xl text-center">
                <Award className="h-8 w-8 text-luxury-gold mx-auto mb-2" />
                <div className="text-2xl font-bold text-luxury-gold">A+</div>
                <div className="text-luxury-platinum/80 text-sm">信用評級</div>
              </div>
            </motion.div>

            {/* Upcoming Events */}
            <motion.div
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: 0.2 }}
              className="luxury-glass p-6 rounded-2xl mb-8"
            >
              <h2 className="text-2xl font-luxury font-semibold text-luxury-gold mb-6">
                即將參與的活動
              </h2>
              <div className="space-y-4">
                {upcomingEvents.map(event => (
                  <div key={event.id} className="flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg">
                    <div>
                      <h3 className="text-luxury-platinum font-medium">{event.name}</h3>
                      <p className="text-luxury-platinum/60 text-sm">
                        {new Date(event.date).toLocaleDateString('zh-TW')}
                      </p>
                    </div>
                    <span className={`px-3 py-1 rounded-full text-xs font-medium border ${getStatusColor(event.status)}`}>
                      {event.status === 'confirmed' ? '已確認' : '待審核'}
                    </span>
                  </div>
                ))}
              </div>
            </motion.div>

            {/* Personal Information */}
            <motion.div
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, delay: 0.3 }}
              className="luxury-glass p-6 rounded-2xl"
            >
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-2xl font-luxury font-semibold text-luxury-gold">
                  個人資訊
                </h2>
                {!isEditing && (
                  <button
                    onClick={handleEdit}
                    className="text-luxury-gold hover:text-luxury-gold/80 transition-colors"
                  >
                    <Edit3 className="h-5 w-5" />
                  </button>
                )}
              </div>

              {isEditing ? (
                <div className="space-y-6">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-luxury-platinum text-sm font-medium mb-2">
                        名字
                      </label>
                      <input
                        type="text"
                        name="firstName"
                        value={editData.firstName}
                        onChange={handleChange}
                        className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors"
                      />
                    </div>
                    <div>
                      <label className="block text-luxury-platinum text-sm font-medium mb-2">
                        姓氏
                      </label>
                      <input
                        type="text"
                        name="lastName"
                        value={editData.lastName}
                        onChange={handleChange}
                        className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors"
                      />
                    </div>
                  </div>

                  <div>
                    <label className="block text-luxury-platinum text-sm font-medium mb-2">
                      職業
                    </label>
                    <input
                      type="text"
                      name="profession"
                      value={editData.profession}
                      onChange={handleChange}
                      className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors"
                    />
                  </div>

                  <div>
                    <label className="block text-luxury-platinum text-sm font-medium mb-2">
                      個人簡介
                    </label>
                    <textarea
                      name="bio"
                      value={editData.bio}
                      onChange={handleChange}
                      rows={3}
                      className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors resize-none"
                    />
                  </div>

                  <div>
                    <label className="block text-luxury-platinum text-sm font-medium mb-2">
                      隱私等級
                    </label>
                    <select
                      name="privacyLevel"
                      value={editData.privacyLevel}
                      onChange={handleChange}
                      className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors"
                    >
                      <option value={1}>1 - 完全公開</option>
                      <option value={2}>2 - 基本資訊公開</option>
                      <option value={3}>3 - 僅會員可見</option>
                      <option value={4}>4 - 僅同等級會員可見</option>
                      <option value={5}>5 - 完全私密</option>
                    </select>
                  </div>

                  <div>
                    <label className="block text-luxury-platinum text-sm font-medium mb-2">
                      興趣愛好
                    </label>
                    <div className="flex space-x-2 mb-3">
                      <input
                        type="text"
                        value={newInterest}
                        onChange={(e) => setNewInterest(e.target.value)}
                        onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addInterest())}
                        className="flex-1 bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-2 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors"
                        placeholder="添加新興趣"
                      />
                      <button
                        type="button"
                        onClick={addInterest}
                        className="px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors"
                      >
                        <Plus className="h-4 w-4" />
                      </button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {editData.interests.map((interest, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-3 py-1 bg-luxury-gold/20 text-luxury-gold text-sm rounded-full"
                        >
                          {interest}
                          <button
                            type="button"
                            onClick={() => removeInterest(interest)}
                            className="ml-2 hover:text-luxury-gold/80"
                          >
                            <X className="h-3 w-3" />
                          </button>
                        </span>
                      ))}
                    </div>
                  </div>

                  <div className="flex space-x-4">
                    <button
                      onClick={handleCancel}
                      className="flex-1 px-4 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
                    >
                      取消
                    </button>
                    <button
                      onClick={handleSave}
                      disabled={isSaving}
                      className="flex-1 luxury-button py-3 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isSaving ? (
                        <>
                          <div className="w-4 h-4 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin mr-2"></div>
                          儲存中...
                        </>
                      ) : (
                        <>
                          <Save className="h-4 w-4 mr-2" />
                          儲存
                        </>
                      )}
                    </button>
                  </div>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-4">
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">姓名</label>
                      <p className="text-luxury-platinum font-medium">
                        {profileData.firstName} {profileData.lastName}
                      </p>
                    </div>
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">電子郵件</label>
                      <p className="text-luxury-platinum font-medium">{profileData.email}</p>
                    </div>
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">職業</label>
                      <p className="text-luxury-platinum font-medium">{profileData.profession}</p>
                    </div>
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">年齡</label>
                      <p className="text-luxury-platinum font-medium">{profileData.age} 歲</p>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">會員等級</label>
                      <p className={`font-medium ${getMembershipColor(profileData.membershipTier)}`}>
                        {profileData.membershipTier}
                      </p>
                    </div>
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">隱私等級</label>
                      <p className="text-luxury-platinum font-medium">
                        Level {profileData.privacyLevel}
                      </p>
                    </div>
                    <div>
                      <label className="text-luxury-platinum/60 text-sm">興趣愛好</label>
                      <div className="flex flex-wrap gap-2 mt-1">
                        {profileData.interests.map((interest, index) => (
                          <span
                            key={index}
                            className="px-2 py-1 bg-luxury-gold/20 text-luxury-gold text-xs rounded-full"
                          >
                            {interest}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </motion.div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ProfilePage