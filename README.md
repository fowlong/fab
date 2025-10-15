# PDF Editor

A proof-of-concept browser-based PDF editor that combines a pdf.js bitmap underlay with a Fabric.js overlay. Edits made in the overlay send structured patch operations to a Rust backend that rewrites the original PDF content streams without rasterisation.

## Project layout

```
pdf-editor/
├── LICENSE
├── README.md
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── types.rs
│       ├── util/
│       │   ├── bbox.rs
│       │   └── matrix.rs
│       └── pdf/
│           ├── mod.rs
│           ├── loader.rs
│           ├── write.rs
│           ├── content.rs
│           ├── extract.rs
│           ├── patch.rs
│           └── fonts/
│               ├── mod.rs
│               ├── shape.rs
│               ├── subset.rs
│               └── embed.rs
├── frontend/
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.ts
│       ├── pdfPreview.ts
│       ├── fabricOverlay.ts
│       ├── api.ts
│       ├── mapping.ts
│       ├── coords.ts
│       ├── types.ts
│       └── styles.css
├── shared/
│   └── schema/
│       ├── ir.schema.json
│       └── patch.schema.json
└── e2e/
    ├── sample.pdf
    └── tests.http
```

## Getting started

### Backend

```
cd backend
cargo run
```

The server listens on `http://localhost:8787` by default.

### Frontend

```
cd frontend
npm install
npm run dev
```

Open the development server at `http://localhost:5173`. Set `FRONTEND_API_BASE` in `.env` to point to the backend if necessary.

## Status

This is an early scaffold containing the foundational modules, type definitions, and API shapes for the MVP described in the repository brief. Many functions currently return `todo!()` or provide stub logic as placeholders for future incremental development.
