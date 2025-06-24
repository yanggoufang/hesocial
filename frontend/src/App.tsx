import { Routes, Route } from 'react-router-dom'
import { motion } from 'framer-motion'
import HomePage from './pages/HomePage'
import EventsPage from './pages/EventsPage'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'
import ProfilePage from './pages/ProfilePage'
import EventDetailPage from './pages/EventDetailPage'
import VVIPPage from './pages/VVIPPage'
import Navbar from './components/Navbar'
import Footer from './components/Footer'

function App() {
  return (
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
        </Routes>
      </motion.main>
      <Footer />
    </div>
  )
}

export default App