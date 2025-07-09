import { Routes, Route } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Suspense, lazy } from 'react';
import { AuthProvider } from './hooks/useAuth';
import Navbar from './components/Navbar';
import Footer from './components/Footer';
import AdminIndicator from './components/AdminIndicator';
import RouteLoader from './components/RouteLoader';
import ErrorBoundary from './components/ErrorBoundary';
import VisitorTracker from './components/VisitorTracker';
import {
  AdminRoute,
  UserRoute,
  VVIPRoute,
  EventManagementRoute,
  UserManagementRoute,
  BackupManagementRoute
} from './components/RouteGuards';

// Lazy load pages with optimized loading
const HomePage = lazy(() => import('./pages/HomePage'));
const LoginPage = lazy(() => import('./pages/LoginPage'));
const EventsPage = lazy(() => import('./pages/EventsPage'));
const EventDetailPage = lazy(() => import('./pages/EventDetailPage'));
const EventRegistration = lazy(() => import('./pages/EventRegistration'));
const RegisterPage = lazy(() => import('./pages/RegisterPage'));
const ProfilePage = lazy(() => import('./pages/ProfilePage'));
const MyRegistrations = lazy(() => import('./pages/MyRegistrations'));
const VVIPPage = lazy(() => import('./pages/VVIPPage'));

// Admin pages (loaded on demand)
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'));
const AnalyticsDashboard = lazy(() => import('./pages/AnalyticsDashboard'));
const BackupManagement = lazy(() => import('./pages/BackupManagement'));
const SystemHealthDashboard = lazy(() => import('./pages/SystemHealthDashboard'));
const UserManagement = lazy(() => import('./pages/UserManagement'));
const SalesManagement = lazy(() => import('./pages/SalesManagement'));

// Event management pages (loaded on demand)
const EventManagement = lazy(() => import('./pages/EventManagement'));
const VenueManagement = lazy(() => import('./pages/VenueManagement'));
const CategoryManagement = lazy(() => import('./pages/CategoryManagement'));
const EventMediaManagement = lazy(() => import('./pages/EventMediaManagement'));

// Social features (loaded on demand)
const EventParticipants = lazy(() => import('./pages/EventParticipants'));
const EventPrivacySettings = lazy(() => import('./pages/EventPrivacySettings'));

// Development pages
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
            <Suspense fallback={<RouteLoader type="general" message="載入頁面中..." />}>
              <Routes>
                {/* Public Routes */}
                <Route path="/" element={<HomePage />} />
                <Route path="/login" element={<LoginPage />} />
                <Route path="/register" element={<RegisterPage />} />
                <Route path="/events" element={<EventsPage />} />
                <Route path="/events/:id" element={<EventDetailPage />} />

                {/* Authenticated User Routes */}
                <Route path="/profile" element={<UserRoute><Suspense fallback={<RouteLoader type="auth" message="載入個人資料中..." />}><ProfilePage /></Suspense></UserRoute>} />
                <Route path="/profile/registrations" element={<UserRoute><Suspense fallback={<RouteLoader type="auth" message="載入註冊記錄中..." />}><MyRegistrations /></Suspense></UserRoute>} />
                <Route path="/events/:eventId/register" element={<UserRoute><Suspense fallback={<RouteLoader type="auth" message="載入註冊頁面中..." />}><EventRegistration /></Suspense></UserRoute>} />
                <Route path="/events/:eventId/participants" element={<UserRoute><Suspense fallback={<RouteLoader type="auth" message="載入參與者資訊中..." />}><EventParticipants /></Suspense></UserRoute>} />
                <Route path="/events/:eventId/privacy-settings" element={<UserRoute><Suspense fallback={<RouteLoader type="auth" message="載入隱私設定中..." />}><EventPrivacySettings /></Suspense></UserRoute>} />
                <Route path="/vvip" element={<VVIPRoute><Suspense fallback={<RouteLoader type="premium" message="載入VIP頁面中..." />}><VVIPPage /></Suspense></VVIPRoute>} />

                {/* Event Management Module */}
                <Route path="/event-mgmt" element={<EventManagementRoute><EventManagement /></EventManagementRoute>} />
                <Route path="/event-mgmt/categories" element={<EventManagementRoute><CategoryManagement /></EventManagementRoute>} />
                <Route path="/event-mgmt/venues" element={<EventManagementRoute><VenueManagement /></EventManagementRoute>} />
                <Route path="/event-mgmt/media/:eventId" element={<EventManagementRoute><EventMediaManagement /></EventManagementRoute>} />

                {/* Admin Module */}
                <Route path="/admin" element={<AdminRoute><Suspense fallback={<RouteLoader type="admin" message="載入管理台中..." />}><AdminDashboard /></Suspense></AdminRoute>} />
                <Route path="/admin/analytics" element={<AdminRoute><Suspense fallback={<RouteLoader type="admin" message="載入分析儀表板中..." />}><AnalyticsDashboard /></Suspense></AdminRoute>} />
                <Route path="/admin/backups" element={<BackupManagementRoute><Suspense fallback={<RouteLoader type="admin" message="載入備份管理中..." />}><BackupManagement /></Suspense></BackupManagementRoute>} />
                <Route path="/admin/users" element={<UserManagementRoute><Suspense fallback={<RouteLoader type="admin" message="載入用戶管理中..." />}><UserManagement /></Suspense></UserManagementRoute>} />
                <Route path="/admin/sales" element={<AdminRoute><Suspense fallback={<RouteLoader type="admin" message="載入銷售管理中..." />}><SalesManagement /></Suspense></AdminRoute>} />
                <Route path="/admin/system" element={<AdminRoute><Suspense fallback={<RouteLoader type="admin" message="載入系統健康檢查中..." />}><SystemHealthDashboard /></Suspense></AdminRoute>} />

                {/* Development Route */}
                <Route path="/access-test" element={<AccessTestPage />} />
              </Routes>
            </Suspense>
          </ErrorBoundary>
        </motion.main>
        <Footer />
        <AdminIndicator position="bottom-right" compact={false} />
        <VisitorTracker showVisitorId={process.env.NODE_ENV === 'development'} />
      </div>
    </AuthProvider>
  );
}

export default App;
