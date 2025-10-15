# PDF Editor

An experimental browser-based PDF editor that combines a Fabric.js overlay with server-side incremental PDF updates.

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

## Getting started

### Backend

```
cd backend
cargo run
```

The development server listens on <http://localhost:8787>.

### Frontend

```
cd frontend
npm install
npm run dev
```

By default Vite serves on <http://localhost:5173> and expects the backend at `/api`.

## Licensing

The repository is distributed under the Apache-2.0 license. Third party client and server dependencies are limited to permissively licensed packages such as Fabric.js, pdf.js, axum, lopdf, harfbuzz-rs and ttf-parser.
