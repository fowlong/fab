import { bootstrapPdfPreview } from "./pdfPreview";
import { bootstrapFabricOverlay } from "./fabricOverlay";
import { ApiClient } from "./api";
import { createEditorLayout } from "./mapping";
import "./styles.css";

const api = new ApiClient((__API_BASE__ as string) ?? "http://localhost:8787");

async function main() {
  const root = document.getElementById("app");
  if (!root) {
    throw new Error("Missing #app root element");
  }

  const layout = createEditorLayout(root);
  const fileInput = layout.uploadInput;

  fileInput.addEventListener("change", async (event) => {
    const target = event.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) {
      return;
    }
    const file = target.files[0];
    const { docId } = await api.open(file);
    const ir = await api.fetchIr(docId);
    const preview = await bootstrapPdfPreview(layout.previewContainer, ir);
    bootstrapFabricOverlay({
      api,
      docId,
      ir,
      preview,
      overlayContainer: layout.overlayContainer
    });
  });
}

void main();
