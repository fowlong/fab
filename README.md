# PDF Editor – Stage 2

This project provides an experimental PDF editor that renders page 0 of a document with `pdf.js`, overlays Fabric.js controllers for editable objects, and rewrites the underlying PDF content streams with incremental saves. The backend is written in Rust (Axum + lopdf) while the frontend is a vanilla TypeScript + Vite application.

## Project layout

```
./
  backend/   # Axum server with PDF parsing, IR extraction, and incremental writer
  frontend/  # Vite TypeScript SPA with pdf.js preview + Fabric overlay
  shared/    # Shared schema notes
  e2e/       # Integration fixtures
```

All dependencies are Apache-2.0 or MIT licensed.

## Stage 2: running & testing

### Backend

```
cd backend
cargo run
```

The service listens on <http://localhost:8787> with the following routes:

* `POST /api/open` – accepts a multipart upload (`file`) or JSON body (`{ "data": "data:application/pdf;base64,..." }`) and returns `{ docId }`.
* `GET /api/ir/:docId` – returns the page‑0 intermediate representation (text + image objects).
* `POST /api/patch/:docId` – accepts transform operations and rewrites the matching content stream, saving the PDF incrementally.
* `GET /api/pdf/:docId` – streams the latest PDF bytes.

### Frontend

```
cd frontend
npm install
npm run dev
```

Navigate to <http://localhost:5173>. The UI renders a toolbar with file picker and download button plus a PDF underlay with Fabric controllers. Upload a single-page PDF and drag / rotate / scale the controllers to apply transforms. Each modification issues a PATCH request and re-renders page 0 with the updated PDF bytes.

### Manual test flow

1. Start backend (`cargo run`) and frontend (`npm run dev`).
2. Load a one-page PDF containing text and an image.
3. Drag the text controller 50 px right and 20 px down, rotate ~10°, scale slightly.
4. Drag/rotate/scale the image controller.
5. Use the download button to fetch the updated PDF. Open it locally to confirm the new placements and that the content stream contains updated `Tm` / `cm` operators while original bytes remain intact (incremental tail appended).

## Contributing

The repository is at an MVP stage. Contributions are welcome via pull request – please keep licences permissive and adhere to the Australian spelling conventions used in comments.
