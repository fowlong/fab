# Backend

The backend is a Rust server built with Axum. It accepts PDF uploads, exposes an
intermediate representation (IR) of page objects, accepts JSON patch requests,
and responds with incremental PDF updates.

## Development

```bash
cargo run
```

The server listens on `http://localhost:8787` by default.
