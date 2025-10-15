import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), 'VITE_');
  return {
    define: {
      __API_BASE__: JSON.stringify(env.VITE_API_BASE_URL ?? 'http://localhost:8787'),
    },
    server: {
      port: 5173,
      host: '0.0.0.0',
    },
  };
});
