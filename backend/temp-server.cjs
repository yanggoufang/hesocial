const express = require('express')
const cors = require('cors')

const app = express()
const PORT = 5000

// Middleware
app.use(cors())
app.use(express.json())

// Mock user data
const mockUsers = [
  {
    id: '1',
    email: 'admin@hesocial.com',
    password: 'admin123',
    firstName: 'Admin',
    lastName: 'User',
    role: 'super_admin',
    membershipTier: 'Black Card',
    isVerified: true,
    verificationStatus: 'approved'
  },
  {
    id: '2',
    email: 'test@example.com',
    password: 'test123',
    firstName: 'Test',
    lastName: 'User',
    role: 'user',
    membershipTier: 'Platinum',
    isVerified: true,
    verificationStatus: 'approved'
  }
]

// Auth routes
app.post('/api/auth/login', (req, res) => {
  const { email, password } = req.body
  
  console.log('Login attempt:', { email, password })
  
  const user = mockUsers.find(u => u.email === email && u.password === password)
  
  if (user) {
    const { password: _, ...userWithoutPassword } = user
    res.json({
      success: true,
      data: {
        user: userWithoutPassword,
        token: 'mock-jwt-token-' + user.id
      }
    })
  } else {
    res.status(401).json({
      success: false,
      error: 'Invalid email or password'
    })
  }
})

app.get('/api/auth/profile', (req, res) => {
  const authHeader = req.headers.authorization
  if (authHeader && authHeader.startsWith('Bearer ')) {
    const token = authHeader.substring(7)
    const userId = token.replace('mock-jwt-token-', '')
    const user = mockUsers.find(u => u.id === userId)
    
    if (user) {
      const { password: _, ...userWithoutPassword } = user
      res.json({
        success: true,
        data: userWithoutPassword
      })
    } else {
      res.status(401).json({
        success: false,
        error: 'Invalid token'
      })
    }
  } else {
    res.status(401).json({
      success: false,
      error: 'No token provided'
    })
  }
})

// Health check
app.get('/api/health', (req, res) => {
  res.json({
    success: true,
    message: 'Temporary server is running',
    timestamp: new Date().toISOString()
  })
})

// API info
app.get('/api', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API - Temporary Server',
    version: '1.0.0',
    features: {
      authentication: '✅ Mock Authentication',
      note: 'This is a temporary server for testing login functionality'
    }
  })
})

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    success: false,
    error: 'Endpoint not found',
    path: req.originalUrl,
    method: req.method
  })
})

app.listen(PORT, () => {
  console.log(`🚀 Temporary HeSocial API Server running on http://localhost:${PORT}`)
  console.log(`📋 Available test accounts:`)
  console.log(`   Admin: admin@hesocial.com / admin123`)
  console.log(`   User:  test@example.com / test123`)
  console.log(`🔍 API Health: http://localhost:${PORT}/api/health`)
})