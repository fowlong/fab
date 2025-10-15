import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  return {
    define: {
      __API_BASE__: JSON.stringify(env.FRONTEND_API_BASE ?? "http://localhost:8787")
    },
    server: {
      port: 5173,
      host: "0.0.0.0"
    }
  };
});
