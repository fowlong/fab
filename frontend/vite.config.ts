import { defineConfig } from "vite";

export default defineConfig({
  server: {
    port: 5173,
    open: true,
  },
  define: {
    __PDF_WORKER_SRC__: JSON.stringify("/pdf.worker.js"),
  },
});
