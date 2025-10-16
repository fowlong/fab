import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const rootDir = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig({
  server: {
    port: 5173,
  },
  preview: {
    port: 5173,
  },
  resolve: {
    alias: {
      fabric: path.resolve(rootDir, 'tests/mocks/fabric.ts'),
      'pdfjs-dist': path.resolve(rootDir, 'tests/mocks/pdfjs-dist.ts'),
      'pdfjs-dist/build/pdf.worker?url': path.resolve(
        rootDir,
        'tests/mocks/pdfjs-dist/build/pdf.worker?url.ts',
      ),
    },
  },
  test: {
    environment: 'node',
    include: ['tests/**/*.spec.ts'],
    setupFiles: ['tests/vitest.setup.ts'],
    deps: {
      inline: ['fabric'],
    },
  },
});
