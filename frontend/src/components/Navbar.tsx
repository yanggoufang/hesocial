import { useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { motion, AnimatePresence } from 'framer-motion'
import { Menu, X, Crown, User, LogOut, Calendar, Settings, Shield, Activity } from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { usePermissions } from '../hooks/useRoleAccess'

const Navbar = () => {
  const [isOpen, setIsOpen] = useState(false)
  const [isUserMenuOpen, setIsUserMenuOpen] = useState(false)
  const location = useLocation()
  const { user, isAuthenticated, logout } = useAuth()
  const { can } = usePermissions()

  const navItems = [
    { name: '首頁', path: '/', icon: null },
    { name: '精選活動', path: '/events', icon: null },
    { name: 'VVIP專區', path: '/vvip', icon: Crown },
  ]

  const isActivePath = (path: string) => location.pathname === path

  return (
    <nav className="fixed top-0 w-full z-50 luxury-glass">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-20">
          <Link to="/" className="flex items-center space-x-2">
            <Crown className="h-8 w-8 text-luxury-gold" />
            <span className="text-2xl font-luxury font-bold text-luxury-gold">
              HeSocial
            </span>
          </Link>

          <div className="hidden md:flex items-center space-x-8">
            {navItems.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center space-x-1 px-4 py-2 rounded-lg transition-all duration-300 ${
                  isActivePath(item.path)
                    ? 'text-luxury-gold bg-luxury-gold/10'
                    : 'text-luxury-platinum hover:text-luxury-gold hover:bg-luxury-gold/5'
                }`}
              >
                {item.icon && <item.icon className="h-4 w-4" />}
                <span>{item.name}</span>
              </Link>
            ))}
          </div>

          <div className="hidden md:flex items-center space-x-4">
            {isAuthenticated ? (
              <div className="relative">
                <button
                  onClick={() => setIsUserMenuOpen(!isUserMenuOpen)}
                  className="flex items-center space-x-2 p-2 rounded-lg hover:bg-luxury-gold/10 transition-colors"
                >
                  <div className="w-8 h-8 bg-luxury-gold rounded-full flex items-center justify-center">
                    <User className="h-4 w-4 text-luxury-midnight-black" />
                  </div>
                </button>

                <AnimatePresence>
                  {isUserMenuOpen && (
                    <motion.div
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, y: 10 }}
                      className="absolute right-0 mt-2 w-48 bg-white/95 backdrop-blur-sm rounded-lg shadow-xl border border-luxury-gold/20 z-[60]"
                    >
                      <Link
                        to="/profile"
                        className="block px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
                        onClick={() => setIsUserMenuOpen(false)}
                      >
                        <User className="inline h-4 w-4 mr-2" />
                        個人檔案
                      </Link>
                      <Link
                        to="/profile/registrations"
                        className="block px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
                        onClick={() => setIsUserMenuOpen(false)}
                      >
                        <Calendar className="inline h-4 w-4 mr-2" />
                        我的報名
                      </Link>
                      
                      {/* Admin Menu Items */}
                      {can.viewAdmin && (
                        <>
                          <div className="border-t border-luxury-gold/20 my-2"></div>
                          <Link
                            to="/admin"
                            className="block px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
                            onClick={() => setIsUserMenuOpen(false)}
                          >
                            <Shield className="inline h-4 w-4 mr-2" />
                            管理後台
                          </Link>
                          <Link
                            to="/event-mgmt"
                            className="block px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
                            onClick={() => setIsUserMenuOpen(false)}
                          >
                            <Settings className="inline h-4 w-4 mr-2" />
                            活動管理
                          </Link>
                          <Link
                            to="/admin/system"
                            className="block px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
                            onClick={() => setIsUserMenuOpen(false)}
                          >
                            <Activity className="inline h-4 w-4 mr-2" />
                            系統健康
                          </Link>
                        </>
                      )}
                      
                      <div className="border-t border-luxury-gold/20 my-2"></div>
                      <button 
                        onClick={() => {
                          logout()
                          setIsUserMenuOpen(false)
                        }}
                        className="block w-full text-left px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
                      >
                        <LogOut className="inline h-4 w-4 mr-2" />
                        登出
                      </button>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            ) : (
              <div className="flex items-center space-x-4">
                <Link
                  to="/login"
                  className="text-luxury-platinum hover:text-luxury-gold transition-colors"
                >
                  登入
                </Link>
                <Link
                  to="/register"
                  className="luxury-button"
                >
                  註冊
                </Link>
              </div>
            )}
          </div>

          <button
            onClick={() => setIsOpen(!isOpen)}
            className="md:hidden p-2 rounded-lg hover:bg-luxury-gold/10 transition-colors"
          >
            {isOpen ? (
              <X className="h-6 w-6 text-luxury-gold" />
            ) : (
              <Menu className="h-6 w-6 text-luxury-gold" />
            )}
          </button>
        </div>
      </div>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="md:hidden luxury-glass border-t border-luxury-gold/20"
          >
            <div className="px-4 py-4 space-y-2">
              {navItems.map((item) => (
                <Link
                  key={item.path}
                  to={item.path}
                  className={`flex items-center space-x-2 px-4 py-3 rounded-lg transition-all duration-300 ${
                    isActivePath(item.path)
                      ? 'text-luxury-gold bg-luxury-gold/10'
                      : 'text-luxury-platinum hover:text-luxury-gold hover:bg-luxury-gold/5'
                  }`}
                  onClick={() => setIsOpen(false)}
                >
                  {item.icon && <item.icon className="h-4 w-4" />}
                  <span>{item.name}</span>
                </Link>
              ))}
              
              {!isAuthenticated && (
                <div className="pt-4 border-t border-luxury-gold/20 space-y-2">
                  <Link
                    to="/login"
                    className="block px-4 py-3 text-luxury-platinum hover:text-luxury-gold transition-colors"
                    onClick={() => setIsOpen(false)}
                  >
                    登入
                  </Link>
                  <Link
                    to="/register"
                    className="block px-4 py-3 bg-luxury-gold text-luxury-midnight-black rounded-lg font-medium transition-colors hover:bg-luxury-gold/90"
                    onClick={() => setIsOpen(false)}
                  >
                    註冊
                  </Link>
                </div>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </nav>
  )
}

export default Navbar