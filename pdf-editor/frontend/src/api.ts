import type { DocumentIR, PatchOp, PatchResponse } from "./types";

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";

export async function loadDocumentIR(file: File): Promise<{ docId: string; ir: DocumentIR }> {
  const form = new FormData();
  form.append("file", file);

  const openResponse = await fetch(`${API_BASE}/api/open`, {
    method: "POST",
    body: form,
  });

  if (!openResponse.ok) {
    throw new Error("Failed to open document");
  }

  const { docId } = (await openResponse.json()) as { docId: string };
  const irResponse = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!irResponse.ok) {
    throw new Error("Failed to fetch IR");
  }
  const ir = (await irResponse.json()) as DocumentIR;
  ir.docId = docId;
  return { docId, ir };
}

export async function sendPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error("Patch request failed");
  }
  return (await response.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!response.ok) {
    throw new Error("Failed to download PDF");
  }
  return await response.blob();
}
