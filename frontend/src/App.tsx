import { Routes, Route } from 'react-router-dom'
import { motion } from 'framer-motion'
import { Suspense, lazy } from 'react'
import { AuthProvider } from './hooks/useAuth'
import Navbar from './components/Navbar'
import Footer from './components/Footer'
import AdminIndicator from './components/AdminIndicator'
import RouteLoader from './components/RouteLoader'
import ErrorBoundary from './components/ErrorBoundary'
import {
  AdminRoute,
  UserRoute,
  VVIPRoute,
  EventManagementRoute,
  UserManagementRoute,
  BackupManagementRoute
} from './components/RouteGuards'

// Lazy load pages for better performance
// Core pages - load immediately
import HomePage from './pages/HomePage'
import LoginPage from './pages/LoginPage'

// Event pages - moderate priority
const EventsPage = lazy(() => import('./pages/EventsPage'))
const EventDetailPage = lazy(() => import('./pages/EventDetailPage'))
const EventRegistration = lazy(() => import('./pages/EventRegistration'))

// User pages - moderate priority
const RegisterPage = lazy(() => import('./pages/RegisterPage'))
const ProfilePage = lazy(() => import('./pages/ProfilePage'))
const MyRegistrations = lazy(() => import('./pages/MyRegistrations'))
const VVIPPage = lazy(() => import('./pages/VVIPPage'))

// Admin pages - low priority (loaded on demand)
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'))
const BackupManagement = lazy(() => import('./pages/BackupManagement'))
const SystemHealthDashboard = lazy(() => import('./pages/SystemHealthDashboard'))

// Event management pages - low priority
const EventManagement = lazy(() => import('./pages/EventManagement'))
const VenueManagement = lazy(() => import('./pages/VenueManagement'))
const CategoryManagement = lazy(() => import('./pages/CategoryManagement'))
const EventMediaManagement = lazy(() => import('./pages/EventMediaManagement'))

// User management - low priority
const UserManagement = lazy(() => import('./pages/UserManagement'))

// Development pages
const AccessTestPage = lazy(() => import('./pages/AccessTestPage'))

function App() {
  return (
    <AuthProvider>
      <div className="min-h-screen bg-luxury-midnight-black text-luxury-platinum">
        <Navbar />
        <motion.main
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6 }}
          className="pt-20"
        >
          <ErrorBoundary>
            <Suspense fallback={<RouteLoader />}>
              <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/events" element={<EventsPage />} />
            <Route path="/events/:id" element={<EventDetailPage />} />
            <Route path="/vvip" element={
              <VVIPRoute>
                <VVIPPage />
              </VVIPRoute>
            } />
            <Route path="/login" element={<LoginPage />} />
            <Route path="/register" element={<RegisterPage />} />
            <Route path="/profile" element={
              <UserRoute>
                <ProfilePage />
              </UserRoute>
            } />
            <Route path="/profile/registrations" element={
              <UserRoute>
                <MyRegistrations />
              </UserRoute>
            } />
            <Route path="/events/:eventId/register" element={
              <UserRoute>
                <EventRegistration />
              </UserRoute>
            } />
            
            {/* Admin Routes - Protected */}
            <Route path="/admin" element={
              <AdminRoute>
                <AdminDashboard />
              </AdminRoute>
            } />
            <Route path="/admin/dashboard" element={
              <AdminRoute>
                <AdminDashboard />
              </AdminRoute>
            } />
            <Route path="/admin/backups" element={
              <BackupManagementRoute>
                <BackupManagement />
              </BackupManagementRoute>
            } />
            <Route path="/admin/users" element={
              <UserManagementRoute>
                <UserManagement />
              </UserManagementRoute>
            } />
            <Route path="/admin/system" element={
              <AdminRoute>
                <SystemHealthDashboard />
              </AdminRoute>
            } />
            
            {/* Event Management Routes - Protected */}
            <Route path="/events/manage" element={
              <EventManagementRoute>
                <EventManagement />
              </EventManagementRoute>
            } />
            <Route path="/events/admin" element={
              <EventManagementRoute>
                <EventManagement />
              </EventManagementRoute>
            } />
            <Route path="/events/venues" element={
              <EventManagementRoute>
                <VenueManagement />
              </EventManagementRoute>
            } />
            <Route path="/events/categories" element={
              <EventManagementRoute>
                <CategoryManagement />
              </EventManagementRoute>
            } />
            <Route path="/events/:eventId/media" element={
              <EventManagementRoute>
                <EventMediaManagement />
              </EventManagementRoute>
            } />
            
            {/* Development Route - Access Control Test */}
            <Route path="/access-test" element={<AccessTestPage />} />
              </Routes>
            </Suspense>
          </ErrorBoundary>
        </motion.main>
        <Footer />
        
        {/* Development Admin Indicator */}
        <AdminIndicator position="bottom-right" compact={false} />
      </div>
    </AuthProvider>
  )
}

export default App