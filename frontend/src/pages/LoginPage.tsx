import { useState } from 'react'
import { motion } from 'framer-motion'
import { Link, useNavigate } from 'react-router-dom'
import { Eye, EyeOff, Crown, Mail, Lock } from 'lucide-react'
import { useAuth } from '../hooks/useAuth'

const LoginPage = () => {
  const navigate = useNavigate()
  const { login, loginWithGoogle, isLoading: authLoading } = useAuth()
  const [formData, setFormData] = useState({
    email: '',
    password: ''
  })
  const [showPassword, setShowPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData(prev => ({
      ...prev,
      [e.target.name]: e.target.value
    }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)
    setError('')

    try {
      const result = await login(formData)
      
      if (result.success) {
        navigate('/')
      } else {
        setError(result.error || '登入失敗，請檢查您的電子郵件和密碼')
      }
    } catch (err) {
      console.error('Login error:', err)
      setError('登入失敗，請檢查您的電子郵件和密碼')
    } finally {
      setIsLoading(false)
    }
  }

  const handleGoogleLogin = () => {
    try {
      loginWithGoogle()
    } catch (err) {
      console.error('Google login error:', err)
      setError('Google 登入失敗，請稍後再試')
    }
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center px-4">
      <div className="max-w-md w-full">
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
              歡迎回來
            </h1>
            <p className="text-luxury-platinum/80">
              登入您的尊榮帳戶
            </p>
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

          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label htmlFor="email" className="block text-luxury-platinum text-sm font-medium mb-2">
                電子郵件
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
                密碼
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
                  placeholder="請輸入您的密碼"
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

            <div className="flex items-center justify-between">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  className="w-4 h-4 text-luxury-gold bg-luxury-midnight-black border-luxury-gold/30 rounded focus:ring-luxury-gold focus:ring-2"
                />
                <span className="ml-2 text-sm text-luxury-platinum/80">記住我</span>
              </label>

              <Link
                to="/forgot-password"
                className="text-sm text-luxury-gold hover:text-luxury-gold/80 transition-colors"
              >
                忘記密碼？
              </Link>
            </div>

            <motion.button
              type="submit"
              disabled={isLoading || authLoading}
              className="w-full luxury-button py-3 disabled:opacity-50 disabled:cursor-not-allowed"
              whileHover={{ scale: (isLoading || authLoading) ? 1 : 1.02 }}
              whileTap={{ scale: (isLoading || authLoading) ? 1 : 0.98 }}
            >
              {(isLoading || authLoading) ? (
                <div className="flex items-center justify-center">
                  <div className="w-5 h-5 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin mr-2"></div>
                  登入中...
                </div>
              ) : (
                '登入'
              )}
            </motion.button>
          </form>

          <div className="mt-8">
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-luxury-gold/20"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-2 bg-luxury-midnight-black text-luxury-platinum/60">或使用</span>
              </div>
            </div>

            <div className="mt-6 grid grid-cols-2 gap-3">
              <motion.button
                type="button"
                onClick={handleGoogleLogin}
                disabled={isLoading || authLoading}
                className="w-full inline-flex justify-center py-2 px-4 border border-luxury-gold/30 rounded-lg shadow-sm bg-luxury-midnight-black/50 text-sm font-medium text-luxury-platinum hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                whileHover={{ scale: (isLoading || authLoading) ? 1 : 1.02 }}
                whileTap={{ scale: (isLoading || authLoading) ? 1 : 0.98 }}
              >
                <svg className="w-5 h-5" viewBox="0 0 24 24">
                  <path fill="currentColor" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                  <path fill="currentColor" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                  <path fill="currentColor" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                  <path fill="currentColor" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                </svg>
                <span className="ml-2">Google</span>
              </motion.button>

              <button className="w-full inline-flex justify-center py-2 px-4 border border-luxury-gold/30 rounded-lg shadow-sm bg-luxury-midnight-black/50 text-sm font-medium text-luxury-platinum/50 cursor-not-allowed" disabled>
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
                </svg>
                <span className="ml-2">LinkedIn (即將推出)</span>
              </button>
            </div>
          </div>

          <div className="mt-8 text-center">
            <p className="text-luxury-platinum/60 text-sm">
              還沒有帳戶？
              <Link
                to="/register"
                className="text-luxury-gold hover:text-luxury-gold/80 font-medium ml-1 transition-colors"
              >
                立即申請加入
              </Link>
            </p>
          </div>
        </motion.div>
      </div>
    </div>
  )
}

export default LoginPage