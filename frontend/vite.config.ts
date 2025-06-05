import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    exclude: ['lucide-react'],
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8084',
        changeOrigin: true,
        secure: false,
      },
      '/ws': {
        target: 'ws://localhost:8084',
        ws: true,
        changeOrigin: true,
      },
    },
  },
  define: {
    'import.meta.env.VITE_API_BASE_URL': JSON.stringify(process.env.VITE_API_BASE_URL || 'http://localhost:8084'),
    'import.meta.env.VITE_WS_BASE_URL': JSON.stringify(process.env.VITE_WS_BASE_URL || 'ws://localhost:8084'),
  },
});
