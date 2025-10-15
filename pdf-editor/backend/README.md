# Backend

The backend is a Rust application built on top of [axum](https://github.com/tokio-rs/axum)
that exposes the REST API needed by the PDF editor frontend. The code base is
organised so that PDF parsing, transformation, and incremental writing logic
can be implemented progressively without blocking the HTTP scaffolding.

## Quick start

```bash
cd backend
cargo run
```

The server listens on `http://localhost:8787` by default. Endpoints are grouped
under the `/api/*` namespace and currently return placeholder responses until
the full PDF pipeline is implemented.

## Project structure

- `src/main.rs` – entry point with route definitions and application state.
- `src/types.rs` – Rust structures mirroring the JSON contracts shared with the
  frontend.
- `src/pdf/*` – modules dedicated to loading, analysing, and writing PDF files.
- `src/util/*` – linear algebra helpers for transformations and bounding boxes.

The modules intentionally expose `TODO` markers so the remaining development
work is clearly signposted.
