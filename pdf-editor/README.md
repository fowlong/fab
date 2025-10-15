# PDF Editor

This repository hosts an experimental browser-based PDF editor. The frontend uses Vite, TypeScript, Fabric.js, and pdf.js to render an interactive editing surface. The backend is written in Rust with axum, lopdf, and harfbuzz-rs to perform structural PDF updates.

## Project structure

```
pdf-editor/
├── LICENSE
├── README.md
├── backend/
│   ├── Cargo.toml
│   └── src/
├── frontend/
│   ├── index.html
│   └── src/
├── shared/
│   └── schema/
└── e2e/
    ├── README.md
    └── tests.http
```

## Getting started

### Backend

```bash
cd backend
cargo run
```

The development server listens on `http://localhost:8787`.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173` in a browser. Set the `VITE_API_BASE` environment variable if the backend runs on a non-default host.

## License

Apache License 2.0. See [LICENSE](LICENSE).
