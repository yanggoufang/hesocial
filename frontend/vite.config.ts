import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    host: '0.0.0.0', // Essential for WSL2 networking
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:5000',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          // Vendor chunks
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'ui-vendor': ['framer-motion', 'lucide-react'],
          
          // Admin chunks (only loaded for admin users)
          'admin-pages': [
            './src/pages/AdminDashboard.tsx',
            './src/pages/BackupManagement.tsx',
            './src/pages/SystemHealthDashboard.tsx',
            './src/pages/UserManagement.tsx'
          ],
          
          // Event management chunks
          'event-management': [
            './src/pages/EventManagement.tsx',
            './src/pages/VenueManagement.tsx',
            './src/pages/CategoryManagement.tsx',
            './src/pages/EventMediaManagement.tsx'
          ],
          
          // User-facing chunks
          'user-pages': [
            './src/pages/EventsPage.tsx',
            './src/pages/EventDetailPage.tsx',
            './src/pages/ProfilePage.tsx',
            './src/pages/MyRegistrations.tsx'
          ]
        }
      }
    },
    chunkSizeWarningLimit: 1000,
  },
})