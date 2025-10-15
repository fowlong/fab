import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "VITE_");
  return {
    define: {
      __API_BASE__: JSON.stringify(env.VITE_API_BASE ?? "http://localhost:8787")
    }
  };
});
