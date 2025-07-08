import { Routes, Route } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Suspense, lazy } from 'react';
import { AuthProvider } from './hooks/useAuth';
import Navbar from './components/Navbar';
import Footer from './components/Footer';
import AdminIndicator from './components/AdminIndicator';
import RouteLoader from './components/RouteLoader';
import ErrorBoundary from './components/ErrorBoundary';
import {
  AdminRoute,
  UserRoute,
  VVIPRoute,
  EventManagementRoute,
  UserManagementRoute,
  BackupManagementRoute
} from './components/RouteGuards';

// Lazy load pages
const HomePage = lazy(() => import('./pages/HomePage'));
const LoginPage = lazy(() => import('./pages/LoginPage'));
const EventsPage = lazy(() => import('./pages/EventsPage'));
const EventDetailPage = lazy(() => import('./pages/EventDetailPage'));
const EventRegistration = lazy(() => import('./pages/EventRegistration'));
const RegisterPage = lazy(() => import('./pages/RegisterPage'));
const ProfilePage = lazy(() => import('./pages/ProfilePage'));
const MyRegistrations = lazy(() => import('./pages/MyRegistrations'));
const VVIPPage = lazy(() => import('./pages/VVIPPage'));
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'));
const BackupManagement = lazy(() => import('./pages/BackupManagement'));
const SystemHealthDashboard = lazy(() => import('./pages/SystemHealthDashboard'));
const EventManagement = lazy(() => import('./pages/EventManagement'));
const VenueManagement = lazy(() => import('./pages/VenueManagement'));
const CategoryManagement = lazy(() => import('./pages/CategoryManagement'));
const EventMediaManagement = lazy(() => import('./pages/EventMediaManagement'));
const UserManagement = lazy(() => import('./pages/UserManagement'));
const SalesManagement = lazy(() => import('./pages/SalesManagement'));
const EventParticipants = lazy(() => import('./pages/EventParticipants'));
const EventPrivacySettings = lazy(() => import('./pages/EventPrivacySettings'));
const AccessTestPage = lazy(() => import('./pages/AccessTestPage'));

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
                {/* Public Routes */}
                <Route path="/" element={<HomePage />} />
                <Route path="/login" element={<LoginPage />} />
                <Route path="/register" element={<RegisterPage />} />
                <Route path="/events" element={<EventsPage />} />
                <Route path="/events/:id" element={<EventDetailPage />} />

                {/* Authenticated User Routes */}
                <Route path="/profile" element={<UserRoute><ProfilePage /></UserRoute>} />
                <Route path="/profile/registrations" element={<UserRoute><MyRegistrations /></UserRoute>} />
                <Route path="/events/:eventId/register" element={<UserRoute><EventRegistration /></UserRoute>} />
                <Route path="/events/:eventId/participants" element={<UserRoute><EventParticipants /></UserRoute>} />
                <Route path="/events/:eventId/privacy-settings" element={<UserRoute><EventPrivacySettings /></UserRoute>} />
                <Route path="/vvip" element={<VVIPRoute><VVIPPage /></VVIPRoute>} />

                {/* Event Management Module */}
                <Route path="/event-mgmt" element={<EventManagementRoute><EventManagement /></EventManagementRoute>} />
                <Route path="/event-mgmt/categories" element={<EventManagementRoute><CategoryManagement /></EventManagementRoute>} />
                <Route path="/event-mgmt/venues" element={<EventManagementRoute><VenueManagement /></EventManagementRoute>} />
                <Route path="/event-mgmt/media/:eventId" element={<EventManagementRoute><EventMediaManagement /></EventManagementRoute>} />

                {/* Admin Module */}
                <Route path="/admin" element={<AdminRoute><AdminDashboard /></AdminRoute>} />
                <Route path="/admin/backups" element={<BackupManagementRoute><BackupManagement /></BackupManagementRoute>} />
                <Route path="/admin/users" element={<UserManagementRoute><UserManagement /></UserManagementRoute>} />
                <Route path="/admin/sales" element={<AdminRoute><SalesManagement /></AdminRoute>} />
                <Route path="/admin/system" element={<AdminRoute><SystemHealthDashboard /></AdminRoute>} />

                {/* Development Route */}
                <Route path="/access-test" element={<AccessTestPage />} />
              </Routes>
            </Suspense>
          </ErrorBoundary>
        </motion.main>
        <Footer />
        <AdminIndicator position="bottom-right" compact={false} />
      </div>
    </AuthProvider>
  );
}

export default App;
