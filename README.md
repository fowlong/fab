# PDF Editor

This repository contains the beginnings of a full-stack PDF editor that keeps
PDF content editable instead of rasterising it. The design centres around a
Fabric.js overlay that mirrors PDF objects so that transforms and text edits can
be written back into the PDF content streams incrementally.

The project is split into the following packages:

* `frontend/` – Vite + TypeScript application that renders each PDF page with
  pdf.js and paints draggable controllers with Fabric.js. For now the preview is
  stubbed but all modules and types are laid out to match the backend API.
* `backend/` – Axum-based Rust service that accepts PDF uploads, exposes the IR
  model, applies patches and emits incremental updates. The parsing and patching
  pipelines are placeholders so that the HTTP contract can stabilise.
* `shared/schema/` – JSON Schemas mirroring the IR and patch payloads to help
  future tooling.
* `e2e/` – Sample PDF and REST Client snippets for quick manual verification.

To run the development servers:

```bash
cargo run --manifest-path backend/Cargo.toml
npm install --prefix frontend
npm run dev --prefix frontend
```

Both the frontend and backend currently surface stub data, providing a scaffold
for implementing the true PDF editing logic iteratively.
