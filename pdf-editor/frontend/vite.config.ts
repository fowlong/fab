import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), 'FRONTEND_');
  return {
    server: {
      port: 5173,
      strictPort: true
    },
    define: {
      __API_BASE__: JSON.stringify(env.FRONTEND_API_BASE || 'http://localhost:8787')
    }
  };
});
