import React from 'react'
import { motion } from 'framer-motion'
import { Shield, Crown, User } from 'lucide-react'

interface RouteLoaderProps {
  type?: 'auth' | 'admin' | 'premium' | 'general'
  message?: string
}

const RouteLoader: React.FC<RouteLoaderProps> = ({ 
  type = 'general', 
  message 
}) => {
  const getIcon = () => {
    switch (type) {
      case 'auth':
        return <User className="h-8 w-8 text-luxury-gold" />
      case 'admin':
        return <Shield className="h-8 w-8 text-luxury-gold" />
      case 'premium':
        return <Crown className="h-8 w-8 text-luxury-gold" />
      default:
        return <div className="w-8 h-8 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin" />
    }
  }

  const getMessage = () => {
    if (message) return message
    
    switch (type) {
      case 'auth':
        return 'Verifying credentials...'
      case 'admin':
        return 'Checking admin access...'
      case 'premium':
        return 'Validating membership...'
      default:
        return 'Loading...'
    }
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="luxury-glass p-8 rounded-2xl text-center"
      >
        <motion.div
          animate={{ 
            rotate: type === 'general' ? 360 : 0,
            scale: [1, 1.1, 1]
          }}
          transition={{ 
            rotate: { duration: 1, repeat: Infinity, ease: "linear" },
            scale: { duration: 2, repeat: Infinity, ease: "easeInOut" }
          }}
          className="flex justify-center mb-4"
        >
          {getIcon()}
        </motion.div>
        
        <motion.p
          animate={{ opacity: [0.5, 1, 0.5] }}
          transition={{ duration: 2, repeat: Infinity, ease: "easeInOut" }}
          className="text-luxury-platinum"
        >
          {getMessage()}
        </motion.p>
      </motion.div>
    </div>
  )
}

export default RouteLoader