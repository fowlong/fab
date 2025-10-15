# PDF Editor

This repository contains a full-stack proof-of-concept for a browser-based PDF editor.
The goal is to provide a standards-compliant workflow for editing PDF content streams
in-place using permissively licensed dependencies only. The project is split into a
TypeScript frontend powered by Vite, Fabric.js, and pdf.js, and a Rust backend built
with axum and lopdf.

## Project layout

```
pdf-editor/
  frontend/   # Vite + Fabric.js client for interactive editing
  backend/    # Axum + lopdf server with incremental PDF rewriting
  shared/     # JSON schemas shared between frontend and backend
  e2e/        # Sample files and HTTP snippets for manual testing
```

Each subdirectory includes its own README content and build tooling instructions.
