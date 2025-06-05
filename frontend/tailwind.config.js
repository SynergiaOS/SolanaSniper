/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        // Trading-specific colors
        'trading': {
          'bull': {
            'primary': '#00C896',
            'light': '#E6F7F3',
            'dark': '#00A67E',
          },
          'bear': {
            'primary': '#FF4757',
            'light': '#FFF1F2',
            'dark': '#E73C4E',
          },
        },
        // Professional status colors
        'status': {
          'active': '#10B981',
          'inactive': '#6B7280',
          'warning': '#F59E0B',
          'error': '#EF4444',
          'info': '#3B82F6',
        },
        // Signal strength colors
        'signal': {
          'strong': '#059669',
          'medium': '#D97706',
          'weak': '#DC2626',
          'very-weak': '#991B1B',
        },
        // Professional grays
        'finance': {
          50: '#F8FAFC',
          100: '#F1F5F9',
          200: '#E2E8F0',
          300: '#CBD5E1',
          400: '#94A3B8',
          500: '#64748B',
          600: '#475569',
          700: '#334155',
          800: '#1E293B',
          900: '#0F172A',
        },
      },
      boxShadow: {
        'trading': '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
        'trading-lg': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
      },
    },
  },
  plugins: [],
};
