# PDF Editor

This project provides a browser-based PDF editor with a Fabric.js overlay for manipulating native PDF objects in real time. The backend is built with Rust and axum while the frontend is a Vite + TypeScript application integrating pdf.js and Fabric.js.

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

* **frontend/** – Vite + TypeScript SPA wiring together pdf.js for rendering and Fabric.js for interactive overlays.
* **backend/** – Rust server exposing REST APIs for opening PDFs, returning an intermediate representation, applying patches, and streaming the current PDF.
* **shared/** – JSON Schemas defining the IR and patch payload formats.
* **e2e/** – Sample assets and REST Client scripts for manual testing.

## Getting Started

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Open <http://localhost:5173> in your browser. Set `VITE_API_BASE` if the backend is hosted elsewhere.

### Backend

```bash
cd backend
cargo run
```

The server listens on `http://localhost:8787` by default.

### End-to-end

Use the sample REST scripts under `e2e/tests.http` with tools like VS Code REST Client or `curl` to exercise the API. A sample PDF is provided at `e2e/sample.pdf` for quick smoke tests.

## Licensing

This repository is licensed under the Apache License 2.0. All bundled dependencies are under permissive licenses (MIT or Apache-2.0).
