import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = __API_BASE__;

export interface OpenResult {
  docId: string;
  ir: DocumentIR;
}

export async function openDocument(file: File): Promise<OpenResult> {
  const form = new FormData();
  form.append('file', file);
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: form
  });
  if (!response.ok) {
    throw new Error(`Failed to open PDF: ${response.statusText}`);
  }
  const { docId } = (await response.json()) as { docId: string };
  const ir = await fetchIR(docId);
  return { docId, ir };
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to load IR: ${response.statusText}`);
  }
  return (await response.json()) as DocumentIR;
}

export async function sendPatch(
  docId: string,
  ops: PatchOperation[]
): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(ops)
  });
  if (!response.ok) {
    throw new Error(`Failed to apply patch: ${response.statusText}`);
  }
  return (await response.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to download PDF: ${response.statusText}`);
  }
  return await response.blob();
}
