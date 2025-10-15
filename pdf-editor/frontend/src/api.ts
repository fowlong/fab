import type { DocumentIr, PatchOperation, PatchResponse } from "./types";

export const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";

export async function openDocument(file: File): Promise<{ docId: string; ir: DocumentIr }>
{
  const form = new FormData();
  form.append("file", file);
  const openResp = await fetch(`${API_BASE}/api/open`, {
    method: "POST",
    body: form,
  });
  if (!openResp.ok) {
    throw new Error(`Failed to open PDF: ${openResp.statusText}`);
  }
  const { docId } = await openResp.json();
  const irResp = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!irResp.ok) {
    throw new Error(`Failed to load IR: ${irResp.statusText}`);
  }
  const ir = (await irResp.json()) as DocumentIr;
  return { docId, ir };
}

export async function fetchIr(docId: string): Promise<DocumentIr> {
  const resp = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!resp.ok) {
    throw new Error(`Failed to fetch IR: ${resp.statusText}`);
  }
  return (await resp.json()) as DocumentIr;
}

export async function applyPatch(
  docId: string,
  ops: PatchOperation[]
): Promise<PatchResponse> {
  const resp = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(ops),
  });
  if (!resp.ok) {
    throw new Error(`Patch failed: ${resp.statusText}`);
  }
  return (await resp.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const resp = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!resp.ok) {
    throw new Error(`Failed to download PDF: ${resp.statusText}`);
  }
  return await resp.blob();
}
