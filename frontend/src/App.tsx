import { Routes, Route } from 'react-router-dom'
import { motion } from 'framer-motion'
import { AuthProvider } from './hooks/useAuth'
import HomePage from './pages/HomePage'
import EventsPage from './pages/EventsPage'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'
import ProfilePage from './pages/ProfilePage'
import EventDetailPage from './pages/EventDetailPage'
import VVIPPage from './pages/VVIPPage'
import AdminDashboard from './pages/AdminDashboard'
import BackupManagement from './pages/BackupManagement'
import EventManagement from './pages/EventManagement'
import VenueManagement from './pages/VenueManagement'
import UserManagement from './pages/UserManagement'
import CategoryManagement from './pages/CategoryManagement'
import EventRegistration from './pages/EventRegistration'
import MyRegistrations from './pages/MyRegistrations'
import SystemHealthDashboard from './pages/SystemHealthDashboard'
import EventMediaManagement from './pages/EventMediaManagement'
import AccessTestPage from './pages/AccessTestPage'
import Navbar from './components/Navbar'
import Footer from './components/Footer'
import AdminIndicator from './components/AdminIndicator'
import {
  AdminRoute,
  SuperAdminRoute,
  UserRoute,
  VVIPRoute,
  EventManagementRoute,
  UserManagementRoute,
  BackupManagementRoute
} from './components/RouteGuards'

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
        </motion.main>
        <Footer />
        
        {/* Development Admin Indicator */}
        <AdminIndicator position="bottom-right" compact={false} />
      </div>
    </AuthProvider>
  )
}

export default App