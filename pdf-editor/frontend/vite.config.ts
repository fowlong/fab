import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: process.env.FRONTEND_API_BASE ?? 'http://localhost:8787',
        changeOrigin: true,
      },
    },
  },
});
