# PDF Editor

This repository contains a permissively licensed proof-of-concept for a visual PDF editor backed by Rust.

## Overview

The project is split into a Vite/TypeScript frontend and an axum-based Rust backend. The frontend renders PDF pages via `pdf.js` and overlays Fabric.js controls that allow a user to transform and edit individual PDF content objects. User edits are sent to the backend where the PDF content streams are rewritten incrementally using `lopdf`.

## Project Structure

```
pdf-editor/
  LICENSE
  README.md
  frontend/
  backend/
  shared/
  e2e/
```

See the inline documentation in each package for implementation notes and TODOs.

## Getting Started

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

Visit `http://localhost:5173` and ensure that the `FRONTEND_API_BASE` environment variable matches the backend origin if the default differs.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](./LICENSE).
