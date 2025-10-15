# PDF Editor (MVP scaffold)

This repository contains an end-to-end prototype for a structured PDF editor
that uses Fabric.js as an interactive overlay and a Rust backend for
incremental PDF updates. The goal is to provide a solid foundation for the
full-featured editor described in the project brief.

## Project layout

```
pdf-editor/
  frontend/      # Vite + TypeScript SPA with Fabric.js and pdf.js
  backend/       # Axum-based API server with lopdf parsing helpers
  shared/        # JSON schemas shared between frontend/backend
  e2e/           # Sample assets and manual API scripts
```

See the README files under each component for build and development
instructions.
