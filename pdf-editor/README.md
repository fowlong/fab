# PDF Editor

A full-stack prototype for a true vector PDF editor. The application allows users to load an existing PDF, render it with `pdf.js`, and manipulate text, image, and path objects using a `fabric.js` overlay. Updates are sent to a Rust backend which rewrites the PDF content streams incrementally so that the document remains vector based.

## Project structure

```
pdf-editor/
├── LICENSE
├── README.md
├── backend
│   ├── Cargo.toml
│   └── src
│       ├── main.rs
│       ├── pdf
│       │   ├── content.rs
│       │   ├── extract.rs
│       │   ├── loader.rs
│       │   ├── mod.rs
│       │   ├── patch.rs
│       │   ├── write.rs
│       │   └── fonts
│       │       ├── embed.rs
│       │       ├── mod.rs
│       │       ├── shape.rs
│       │       └── subset.rs
│       ├── types.rs
│       └── util
│           ├── bbox.rs
│           └── matrix.rs
├── e2e
│   ├── sample.pdf
│   └── tests.http
├── frontend
│   ├── index.html
│   ├── package.json
│   ├── styles.css
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── src
│       ├── api.ts
│       ├── fabricOverlay.ts
│       ├── main.ts
│       ├── mapping.ts
│       ├── pdfPreview.ts
│       ├── coords.ts
│       └── types.ts
└── shared
    └── schema
        ├── ir.schema.json
        └── patch.schema.json
```

## Getting started

### Backend

```bash
cd backend
cargo run
```

The server starts on <http://localhost:8787>.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The development server starts on <http://localhost:5173>.

Set the `VITE_API_BASE` environment variable if the backend runs on a different origin.
