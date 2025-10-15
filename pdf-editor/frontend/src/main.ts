import "./styles.css";
import { initPdfPreview } from "./pdfPreview";
import { initFabricOverlay } from "./fabricOverlay";
import { loadInitialDocument } from "./api";

async function bootstrap() {
  const root = document.getElementById("app");
  if (!root) {
    throw new Error("Missing app container");
  }

  root.innerHTML = `
    <main class="layout">
      <section class="sidebar" id="sidebar"></section>
      <section class="editor">
        <div class="toolbar" id="toolbar"></div>
        <div class="page-container" id="page-container"></div>
      </section>
    </main>
  `;

  const doc = await loadInitialDocument();
  const preview = await initPdfPreview(doc, document.getElementById("page-container"));
  initFabricOverlay(doc, preview);
}

bootstrap().catch((err) => {
  console.error("Failed to initialise application", err);
});
