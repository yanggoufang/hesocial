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
import Navbar from './components/Navbar'
import Footer from './components/Footer'

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
            <Route path="/vvip" element={<VVIPPage />} />
            <Route path="/login" element={<LoginPage />} />
            <Route path="/register" element={<RegisterPage />} />
            <Route path="/profile" element={<ProfilePage />} />
            {/* Admin Routes */}
            <Route path="/admin" element={<AdminDashboard />} />
            <Route path="/admin/dashboard" element={<AdminDashboard />} />
            <Route path="/admin/backups" element={<BackupManagement />} />
            <Route path="/admin/users" element={<UserManagement />} />
            
            {/* Event Management Routes (Separate System) */}
            <Route path="/events/manage" element={<EventManagement />} />
            <Route path="/events/admin" element={<EventManagement />} />
            <Route path="/events/venues" element={<VenueManagement />} />
            <Route path="/events/categories" element={<CategoryManagement />} />
          </Routes>
        </motion.main>
        <Footer />
      </div>
    </AuthProvider>
  )
}

export default App