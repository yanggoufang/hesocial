/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        luxury: {
          gold: '#D4AF37',
          'deep-blue': '#1B2951',
          platinum: '#E5E4E2',
          champagne: '#F7E7CE',
          'midnight-black': '#0C0C0C',
        },
      },
      fontFamily: {
        'luxury': ['Playfair Display', 'serif'],
        'sans': ['Inter', 'system-ui', 'sans-serif'],
      },
      animation: {
        'luxury-fade': 'luxuryFade 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
        'luxury-slide': 'luxurySlide 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      },
      keyframes: {
        luxuryFade: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        luxurySlide: {
          '0%': { transform: 'translateX(-20px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
      },
      backdropBlur: {
        'luxury': '20px',
      },
    },
  },
  plugins: [],
}