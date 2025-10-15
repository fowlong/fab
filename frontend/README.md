# Frontend

The frontend is a Vite + TypeScript single-page app that renders PDF pages with `pdf.js` and overlays editable controllers with `fabric.js`.

## Commands

```bash
npm install     # install dependencies
npm run dev     # start Vite dev server on http://localhost:5173
npm run build   # build production assets
```

Set `VITE_API_BASE` to point to the Rust backend (defaults to `http://localhost:8787`).
