import type { DocumentIr, PatchOperation, PatchResponse } from "./types";

const DEFAULT_API_BASE = "http://localhost:8787";
const apiBase = (import.meta.env.VITE_API_BASE as string | undefined) ?? DEFAULT_API_BASE;

export async function openDocument(
  file: File
): Promise<{ docId: string; ir: DocumentIr; pdfBytes: Uint8Array }> {
  const pdfBytes = new Uint8Array(await file.arrayBuffer());
  const base64 = toBase64(pdfBytes);
  const response = await fetch(`${apiBase}/api/open`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ pdfBase64: base64 }),
  });
  if (!response.ok) {
    throw new Error(`Failed to open document: ${response.statusText}`);
  }
  const { docId } = (await response.json()) as { docId: string };
  const ir = await fetchIr(docId);
  return { docId, ir, pdfBytes };
}

export async function fetchIr(docId: string): Promise<DocumentIr> {
  const response = await fetch(`${apiBase}/api/ir/${docId}`);
  if (!response.ok) {
    throw new Error(`Failed to load IR: ${response.statusText}`);
  }
  return (await response.json()) as DocumentIr;
}

export async function sendPatch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  const response = await fetch(`${apiBase}/api/patch/${docId}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error(`Failed to apply patch: ${response.statusText}`);
  }
  return (await response.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${apiBase}/api/pdf/${docId}`);
  if (!response.ok) {
    throw new Error(`Failed to download PDF: ${response.statusText}`);
  }
  return await response.blob();
}

function toBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    const chunk = bytes.subarray(offset, offset + chunkSize);
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}
