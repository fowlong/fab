# PDF Editor Prototype

This repository contains a full-stack prototype for a structured PDF editor.
It is implemented according to the provided project specification with:

- **Frontend**: Vite + TypeScript, Fabric.js overlay, pdf.js preview.
- **Backend**: Rust (axum) with PDF parsing/writing implemented using `lopdf` and
  supporting crates (`harfbuzz-rs`, `ttf-parser`).
- **Shared schemas**: JSON Schema contracts for the intermediate representation
  and patch protocol.
- **E2E fixtures**: Sample PDF and REST examples for manual testing.

The working tree includes scaffolding, documentation, and stubbed code paths to
support incremental development of the full editing pipeline.

See [`pdf-editor/README.md`](pdf-editor/README.md) for detailed usage and
architecture notes.
