# PDF Editor

An end-to-end PDF editor that renders a PDF with pdf.js and overlays interactive Fabric.js controls. User manipulations are translated into PDF content stream edits on the server so the document remains fully vector-based.

## Project layout

```
pdf-editor/
  LICENSE
  README.md
  frontend/
  backend/
  shared/
  e2e/
```

The frontend is a Vite + TypeScript single-page application that uses `pdfjs-dist` to render PDF page previews and Fabric.js to render controllers on top of those previews. The backend is an Axum service written in Rust that parses PDF content streams with `lopdf` and applies incremental updates.

## Development

### Backend

```bash
cd backend
cargo run
```

The server listens on `http://localhost:8787` by default.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173` in a browser. Set `VITE_API_BASE` to the backend URL if the frontend and backend are served from different origins.

## License

Apache-2.0. See [LICENSE](./LICENSE).
