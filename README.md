# PDF Editor (MVP scaffold)

This repository hosts a proof-of-concept PDF editor that combines a Fabric.js interaction layer with pdf.js rendering and a Rust backend built on axum. The goal is to enable true PDF editing with incremental saves rather than raster overlays.

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

The detailed layout is described in the project plan and mirrored in the directory tree.

## Getting started

### Prerequisites

- Node.js 18+
- npm 9+
- Rust toolchain (stable) with `cargo`

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The development server listens on <http://localhost:5173>. Set `VITE_API_BASE` if the backend runs on a custom port.

### Backend

```bash
cd backend
cargo run
```

The backend server binds to <http://localhost:8787> in development mode.

## License

Apache License 2.0. See [LICENSE](LICENSE).
