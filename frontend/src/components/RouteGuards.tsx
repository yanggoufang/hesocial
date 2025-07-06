import React from 'react'
import ProtectedRoute from './ProtectedRoute'

// Admin Route Guard - Requires admin or super_admin role
export const AdminRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute requireRole="admin" fallbackPath="/login">
      {children}
    </ProtectedRoute>
  )
}

// Super Admin Route Guard - Requires super_admin role only
export const SuperAdminRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute requireRole="super_admin" fallbackPath="/admin">
      {children}
    </ProtectedRoute>
  )
}

// User Route Guard - Requires basic authentication
export const UserRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute requireAuth={true} fallbackPath="/login">
      {children}
    </ProtectedRoute>
  )
}

// Verified User Route Guard - Requires verified account
export const VerifiedUserRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireAuth={true} 
      requireVerification={true} 
      fallbackPath="/profile"
    >
      {children}
    </ProtectedRoute>
  )
}

// Diamond Member Route Guard - Requires Diamond tier or higher
export const DiamondRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireAuth={true} 
      requireMembership="Diamond" 
      fallbackPath="/events"
    >
      {children}
    </ProtectedRoute>
  )
}

// Black Card Route Guard - Requires Black Card tier
export const BlackCardRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireAuth={true} 
      requireMembership="Black Card" 
      fallbackPath="/events"
    >
      {children}
    </ProtectedRoute>
  )
}

// VVIP Route Guard - Requires Diamond+ membership and verification
export const VVIPRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireAuth={true} 
      requireMembership="Diamond" 
      requireVerification={true}
      fallbackPath="/events"
    >
      {children}
    </ProtectedRoute>
  )
}

// Event Management Route Guard - Admin access for event management
export const EventManagementRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireRole="admin" 
      fallbackPath="/login"
    >
      {children}
    </ProtectedRoute>
  )
}

// User Management Route Guard - Admin access for user management
export const UserManagementRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireRole="admin" 
      fallbackPath="/admin"
    >
      {children}
    </ProtectedRoute>
  )
}

// Backup Management Route Guard - Super admin only
export const BackupManagementRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <ProtectedRoute 
      requireRole="super_admin" 
      fallbackPath="/admin"
    >
      {children}
    </ProtectedRoute>
  )
}