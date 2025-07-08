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
        manualChunks: (id) => {
          // Vendor chunks
          if (id.includes('node_modules')) {
            if (id.includes('react') || id.includes('react-dom') || id.includes('react-router')) {
              return 'react-vendor';
            }
            if (id.includes('framer-motion') || id.includes('lucide-react')) {
              return 'ui-vendor';
            }
            return 'vendor';
          }
          
          // Admin chunks (only loaded for admin users)
          if (id.includes('pages/Admin') || id.includes('pages/Backup') || 
              id.includes('pages/SystemHealth') || id.includes('pages/UserManagement') ||
              id.includes('pages/SalesManagement')) {
            return 'admin-pages';
          }
          
          // Event management chunks
          if (id.includes('pages/Event') && id.includes('Management')) {
            return 'event-management';
          }
          
          // Social features chunks
          if (id.includes('pages/EventParticipants') || id.includes('pages/EventPrivacy')) {
            return 'social-features';
          }
          
          // User-facing chunks
          if (id.includes('pages/Events') || id.includes('pages/Profile') || 
              id.includes('pages/MyRegistrations')) {
            return 'user-pages';
          }
          
          // Auth chunks
          if (id.includes('pages/Login') || id.includes('pages/Register')) {
            return 'auth-pages';
          }
        }
      }
    },
    chunkSizeWarningLimit: 1000,
  },
})