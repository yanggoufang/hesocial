import { useState } from 'react'
import { motion } from 'framer-motion'
import { Link, useNavigate } from 'react-router-dom'
import { Eye, EyeOff, Crown, Mail, Lock, User, Briefcase, DollarSign, Plus, X } from 'lucide-react'
import { useAuth } from '../hooks/useAuth'

const RegisterPage = () => {
  const navigate = useNavigate()
  const { register } = useAuth()
  const [currentStep, setCurrentStep] = useState(1)
  const [formData, setFormData] = useState({
    email: '',
    password: '',
    confirmPassword: '',
    firstName: '',
    lastName: '',
    age: '',
    profession: '',
    annualIncome: '',
    netWorth: '',
    membershipTier: 'Platinum',
    bio: '',
    interests: [] as string[]
  })
  
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')
  const [newInterest, setNewInterest] = useState('')

  const membershipTiers = [
    {
      value: 'Platinum',
      label: 'Platinum',
      price: 'NT$50,000/年',
      description: '基礎尊榮會員，享受精選活動與服務'
    },
    {
      value: 'Diamond',
      label: 'Diamond',
      price: 'NT$120,000/年',
      description: 'VIP會員專屬活動與專人顧問服務'
    },
    {
      value: 'Black Card',
      label: 'Black Card',
      price: '邀請制',
      description: '最高等級會員，享受所有獨家特權'
    }
  ]

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    setFormData(prev => ({
      ...prev,
      [e.target.name]: e.target.value
    }))
  }

  const addInterest = () => {
    if (newInterest.trim() && !formData.interests.includes(newInterest.trim()) && formData.interests.length < 10) {
      setFormData(prev => ({
        ...prev,
        interests: [...prev.interests, newInterest.trim()]
      }))
      setNewInterest('')
    }
  }

  const removeInterest = (interest: string) => {
    setFormData(prev => ({
      ...prev,
      interests: prev.interests.filter(i => i !== interest)
    }))
  }

  const validateStep1 = () => {
    if (!formData.email || !formData.password || !formData.confirmPassword) {
      setError('請填寫所有必填欄位')
      return false
    }
    if (formData.password !== formData.confirmPassword) {
      setError('密碼確認不相符')
      return false
    }
    if (formData.password.length < 8) {
      setError('密碼長度至少需要8個字元')
      return false
    }
    return true
  }

  const validateStep2 = () => {
    if (!formData.firstName || !formData.lastName || !formData.age || !formData.profession) {
      setError('請填寫所有必填欄位')
      return false
    }
    if (parseInt(formData.age) < 18 || parseInt(formData.age) > 100) {
      setError('年齡必須在18-100歲之間')
      return false
    }
    return true
  }

  const validateStep3 = () => {
    if (!formData.annualIncome || !formData.netWorth) {
      setError('請填寫收入與資產資訊')
      return false
    }
    if (parseInt(formData.annualIncome) < 5000000) {
      setError('年收入需達500萬以上才符合申請資格')
      return false
    }
    if (parseInt(formData.netWorth) < 30000000) {
      setError('淨資產需達3000萬以上才符合申請資格')
      return false
    }
    if (formData.interests.length === 0) {
      setError('請至少添加一個興趣')
      return false
    }
    return true
  }

  const nextStep = () => {
    setError('')
    
    if (currentStep === 1 && !validateStep1()) return
    if (currentStep === 2 && !validateStep2()) return
    if (currentStep === 3 && !validateStep3()) return
    
    if (currentStep < 3) {
      setCurrentStep(currentStep + 1)
    } else {
      handleSubmit()
    }
  }

  const prevStep = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1)
      setError('')
    }
  }

  const handleSubmit = async () => {
    setIsLoading(true)
    setError('')

    try {
      const registrationData = {
        email: formData.email,
        password: formData.password,
        firstName: formData.firstName,
        lastName: formData.lastName,
        age: parseInt(formData.age),
        profession: formData.profession,
        annualIncome: parseInt(formData.annualIncome),
        netWorth: parseInt(formData.netWorth),
        bio: formData.bio,
        interests: formData.interests
      }

      const result = await register(registrationData)
      
      if (result.success) {
        navigate('/dashboard', { 
          state: { message: '註冊成功！歡迎加入 HeSocial。' }
        })
      } else {
        setError(result.error || '註冊失敗，請稍後再試')
      }
    } catch (err) {
      console.error('Registration error:', err)
      setError('註冊失敗，請稍後再試')
    } finally {
      setIsLoading(false)
    }
  }

  const renderStep1 = () => (
    <div className="space-y-6">
      <div>
        <label htmlFor="email" className="block text-luxury-platinum text-sm font-medium mb-2">
          電子郵件 *
        </label>
        <div className="relative">
          <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
          <input
            type="email"
            id="email"
            name="email"
            value={formData.email}
            onChange={handleChange}
            required
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="請輸入您的電子郵件"
          />
        </div>
      </div>

      <div>
        <label htmlFor="password" className="block text-luxury-platinum text-sm font-medium mb-2">
          密碼 *
        </label>
        <div className="relative">
          <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
          <input
            type={showPassword ? 'text' : 'password'}
            id="password"
            name="password"
            value={formData.password}
            onChange={handleChange}
            required
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 pr-12 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="至少8個字元，包含大小寫字母、數字和特殊符號"
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-1/2 transform -translate-y-1/2 text-luxury-gold hover:text-luxury-gold/80 transition-colors"
          >
            {showPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
          </button>
        </div>
      </div>

      <div>
        <label htmlFor="confirmPassword" className="block text-luxury-platinum text-sm font-medium mb-2">
          確認密碼 *
        </label>
        <div className="relative">
          <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
          <input
            type={showConfirmPassword ? 'text' : 'password'}
            id="confirmPassword"
            name="confirmPassword"
            value={formData.confirmPassword}
            onChange={handleChange}
            required
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 pr-12 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="請再次輸入密碼"
          />
          <button
            type="button"
            onClick={() => setShowConfirmPassword(!showConfirmPassword)}
            className="absolute right-3 top-1/2 transform -translate-y-1/2 text-luxury-gold hover:text-luxury-gold/80 transition-colors"
          >
            {showConfirmPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
          </button>
        </div>
      </div>
    </div>
  )

  const renderStep2 = () => (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label htmlFor="firstName" className="block text-luxury-platinum text-sm font-medium mb-2">
            名字 *
          </label>
          <div className="relative">
            <User className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
            <input
              type="text"
              id="firstName"
              name="firstName"
              value={formData.firstName}
              onChange={handleChange}
              required
              className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
              placeholder="名字"
            />
          </div>
        </div>

        <div>
          <label htmlFor="lastName" className="block text-luxury-platinum text-sm font-medium mb-2">
            姓氏 *
          </label>
          <input
            type="text"
            id="lastName"
            name="lastName"
            value={formData.lastName}
            onChange={handleChange}
            required
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="姓氏"
          />
        </div>
      </div>

      <div>
        <label htmlFor="age" className="block text-luxury-platinum text-sm font-medium mb-2">
          年齡 *
        </label>
        <input
          type="number"
          id="age"
          name="age"
          value={formData.age}
          onChange={handleChange}
          required
          min="18"
          max="100"
          className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
          placeholder="請輸入年齡"
        />
      </div>

      <div>
        <label htmlFor="profession" className="block text-luxury-platinum text-sm font-medium mb-2">
          職業 *
        </label>
        <div className="relative">
          <Briefcase className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
          <input
            type="text"
            id="profession"
            name="profession"
            value={formData.profession}
            onChange={handleChange}
            required
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="例：企業執行長、投資銀行家、醫師"
          />
        </div>
      </div>
    </div>
  )

  const renderStep3 = () => (
    <div className="space-y-6">
      <div>
        <label htmlFor="annualIncome" className="block text-luxury-platinum text-sm font-medium mb-2">
          年收入 (新台幣萬元) *
        </label>
        <div className="relative">
          <DollarSign className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
          <input
            type="number"
            id="annualIncome"
            name="annualIncome"
            value={formData.annualIncome}
            onChange={handleChange}
            required
            min="5000000"
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="最低500萬"
          />
        </div>
        <p className="text-luxury-platinum/60 text-xs mt-1">申請資格：年收入需達新台幣500萬以上</p>
      </div>

      <div>
        <label htmlFor="netWorth" className="block text-luxury-platinum text-sm font-medium mb-2">
          淨資產 (新台幣萬元) *
        </label>
        <div className="relative">
          <DollarSign className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold" />
          <input
            type="number"
            id="netWorth"
            name="netWorth"
            value={formData.netWorth}
            onChange={handleChange}
            required
            min="30000000"
            className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="最低3000萬"
          />
        </div>
        <p className="text-luxury-platinum/60 text-xs mt-1">申請資格：淨資產需達新台幣3000萬以上</p>
      </div>

      <div>
        <label htmlFor="membershipTier" className="block text-luxury-platinum text-sm font-medium mb-2">
          會員等級 *
        </label>
        <div className="space-y-3">
          {membershipTiers.map(tier => (
            <label key={tier.value} className="flex items-start space-x-3 cursor-pointer">
              <input
                type="radio"
                name="membershipTier"
                value={tier.value}
                checked={formData.membershipTier === tier.value}
                onChange={handleChange}
                className="mt-1 w-4 h-4 text-luxury-gold bg-luxury-midnight-black border-luxury-gold/30 focus:ring-luxury-gold focus:ring-2"
              />
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <span className="text-luxury-gold font-medium">{tier.label}</span>
                  <span className="text-luxury-platinum/80 text-sm">{tier.price}</span>
                </div>
                <p className="text-luxury-platinum/60 text-sm">{tier.description}</p>
              </div>
            </label>
          ))}
        </div>
      </div>

      <div>
        <label htmlFor="interests" className="block text-luxury-platinum text-sm font-medium mb-2">
          興趣愛好 *
        </label>
        <div className="flex space-x-2 mb-3">
          <input
            type="text"
            value={newInterest}
            onChange={(e) => setNewInterest(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addInterest())}
            className="flex-1 bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-2 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors"
            placeholder="添加興趣（例：藝術收藏、高爾夫、紅酒品鑑）"
            maxLength={50}
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
          {formData.interests.map((interest, index) => (
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
        <p className="text-luxury-platinum/60 text-xs mt-1">請至少添加1個興趣，最多10個</p>
      </div>

      <div>
        <label htmlFor="bio" className="block text-luxury-platinum text-sm font-medium mb-2">
          個人簡介 (選填)
        </label>
        <textarea
          id="bio"
          name="bio"
          value={formData.bio}
          onChange={handleChange}
          rows={3}
          maxLength={500}
          className="w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors resize-none"
          placeholder="請簡述您的背景與興趣（選填，最多500字）"
        />
        <p className="text-luxury-platinum/60 text-xs mt-1">{formData.bio.length}/500</p>
      </div>
    </div>
  )

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-8 px-4">
      <div className="max-w-2xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="luxury-glass p-8 rounded-2xl"
        >
          <div className="text-center mb-8">
            <div className="flex items-center justify-center mb-4">
              <Crown className="h-12 w-12 text-luxury-gold" />
            </div>
            <h1 className="text-3xl font-luxury font-bold text-luxury-gold mb-2">
              加入 HeSocial
            </h1>
            <p className="text-luxury-platinum/80">
              申請成為尊榮會員，開啟您的頂級社交之旅
            </p>
          </div>

          {/* Progress Steps */}
          <div className="flex items-center justify-center mb-8">
            {[1, 2, 3].map((step) => (
              <div key={step} className="flex items-center">
                <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                  step <= currentStep 
                    ? 'bg-luxury-gold text-luxury-midnight-black' 
                    : 'bg-luxury-gold/20 text-luxury-gold'
                }`}>
                  {step}
                </div>
                {step < 3 && (
                  <div className={`w-12 h-0.5 mx-2 ${
                    step < currentStep ? 'bg-luxury-gold' : 'bg-luxury-gold/20'
                  }`}></div>
                )}
              </div>
            ))}
          </div>

          <div className="text-center mb-6">
            <h2 className="text-xl font-luxury text-luxury-gold">
              {currentStep === 1 && '步驟 1: 帳戶設定'}
              {currentStep === 2 && '步驟 2: 個人資訊'}
              {currentStep === 3 && '步驟 3: 會員資格'}
            </h2>
          </div>

          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm"
            >
              {error}
            </motion.div>
          )}

          <form onSubmit={(e) => e.preventDefault()}>
            {currentStep === 1 && renderStep1()}
            {currentStep === 2 && renderStep2()}
            {currentStep === 3 && renderStep3()}

            <div className="flex justify-between mt-8">
              <button
                type="button"
                onClick={prevStep}
                disabled={currentStep === 1}
                className="px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                上一步
              </button>

              <motion.button
                type="button"
                onClick={nextStep}
                disabled={isLoading}
                className="px-6 py-3 luxury-button disabled:opacity-50 disabled:cursor-not-allowed"
                whileHover={{ scale: isLoading ? 1 : 1.02 }}
                whileTap={{ scale: isLoading ? 1 : 0.98 }}
              >
                {isLoading ? (
                  <div className="flex items-center">
                    <div className="w-5 h-5 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin mr-2"></div>
                    提交申請中...
                  </div>
                ) : currentStep === 3 ? (
                  '提交申請'
                ) : (
                  '下一步'
                )}
              </motion.button>
            </div>
          </form>

          <div className="mt-8 text-center">
            <p className="text-luxury-platinum/60 text-sm">
              已有帳戶？
              <Link
                to="/login"
                className="text-luxury-gold hover:text-luxury-gold/80 font-medium ml-1 transition-colors"
              >
                立即登入
              </Link>
            </p>
          </div>
        </motion.div>
      </div>
    </div>
  )
}

export default RegisterPage