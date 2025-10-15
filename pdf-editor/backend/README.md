# Backend

The backend is an [axum](https://github.com/tokio-rs/axum)-based Rust service that parses PDFs to an intermediate representation (IR), applies JSON-style patches, and serialises incremental updates back to PDF.

## Key crates

- [`axum`](https://crates.io/crates/axum) (MIT/Apache-2.0 dual license)
- [`tokio`](https://crates.io/crates/tokio) (MIT)
- [`serde`/`serde_json`](https://serde.rs/) (MIT/Apache-2.0)
- [`lopdf`](https://crates.io/crates/lopdf) (MIT/Apache-2.0)
- [`harfbuzz_rs`](https://crates.io/crates/harfbuzz_rs) (MIT)
- [`ttf-parser`](https://crates.io/crates/ttf-parser) (MIT/Apache-2.0)

## Development

```
cargo run --bin server
```

The application exposes the following routes (handler stubs are implemented in `src/main.rs`):

- `POST /api/open`
- `GET /api/ir/:docId`
- `POST /api/patch/:docId`
- `GET /api/pdf/:docId`

Each handler delegates to modules within `src/pdf` to load documents, extract IR, apply patches, and write incremental updates.

## Status

Modules under `src/pdf` are scaffolds containing type definitions, trait boundaries, and detailed documentation comments describing the intended logic. The scaffolding is useful for parallel development while keeping architectural decisions documented in code.
