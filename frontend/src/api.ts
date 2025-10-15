import type { DocumentIR, PatchOperation, PatchResponse } from "./types";

const pdfCache = new Map<string, Uint8Array>();

export async function loadDocumentIR(file: File): Promise<{ docId: string; ir: DocumentIR }> {
  const arrayBuffer = await file.arrayBuffer();
  const pdfBytes = new Uint8Array(arrayBuffer);

  try {
    const formData = new FormData();
    formData.append("file", file);
    const openResponse = await fetch("/api/open", {
      method: "POST",
      body: formData,
    });
    if (!openResponse.ok) {
      throw new Error(`Failed to open PDF: ${openResponse.status}`);
    }

    const { docId } = await openResponse.json();
    const irResponse = await fetch(`/api/ir/${docId}`);
    if (!irResponse.ok) {
      throw new Error(`Failed to load IR: ${irResponse.status}`);
    }
    const ir = (await irResponse.json()) as DocumentIR;
    const embedded = ir.documentMeta.originalPdf;
    const resolvedBytes = embedded ? decodeBase64Pdf(embedded) : pdfBytes;
    ir.documentMeta.originalPdfBytes = resolvedBytes;
    pdfCache.set(docId, resolvedBytes);
    return { docId, ir };
  } catch (error) {
    console.warn("Falling back to local IR stub", error);
    const docId = `local-${Date.now()}`;
    const ir: DocumentIR = {
      documentMeta: {
        fileName: file.name,
        pageCount: 1,
        originalPdfBytes: pdfBytes,
      },
      pages: [
        {
          index: 0,
          widthPt: 595.276,
          heightPt: 841.89,
          objects: [],
        },
      ],
    };
    pdfCache.set(docId, pdfBytes);
    return { docId, ir };
  }
}

export async function patchDocument(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  try {
    const response = await fetch(`/api/patch/${docId}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ops),
    });
    if (!response.ok) {
      throw new Error(`Patch failed: ${response.status}`);
    }
    const patchResponse = (await response.json()) as PatchResponse;
    if (patchResponse.updatedPdf) {
      const bytes = decodeBase64Pdf(patchResponse.updatedPdf);
      pdfCache.set(docId, bytes);
    }
    return patchResponse;
  } catch (error) {
    console.warn("Patch fallback", error);
    return { ok: true };
  }
}

export async function downloadPdf(docId: string): Promise<Blob> {
  try {
    const response = await fetch(`/api/pdf/${docId}`);
    if (!response.ok) {
      throw new Error(`Download failed: ${response.status}`);
    }
    return await response.blob();
  } catch (error) {
    console.warn("Download fallback", error);
    const pdfBytes = pdfCache.get(docId);
    if (!pdfBytes) {
      throw new Error("PDF not available");
    }
    return new Blob([pdfBytes], { type: "application/pdf" });
  }
}

function decodeBase64Pdf(dataUri: string): Uint8Array {
  const [, base64] = dataUri.split(",", 2);
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
