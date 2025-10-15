import type { DocumentIR, PatchOp, PatchResponse } from './types';

const API_BASE = import.meta.env.FRONTEND_API_BASE ?? 'http://localhost:8787';

export async function openDocument(file: File): Promise<string> {
  const body = new FormData();
  body.append('file', file);
  const res = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body
  });
  if (!res.ok) {
    throw new Error(`Failed to open PDF: ${res.statusText}`);
  }
  const data = await res.json();
  return data.docId;
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const res = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!res.ok) {
    throw new Error(`Failed to load IR: ${res.statusText}`);
  }
  return res.json();
}

export async function sendPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const res = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops)
  });
  if (!res.ok) {
    throw new Error(`Failed to apply patch: ${res.statusText}`);
  }
  return res.json();
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error(`Failed to download PDF: ${res.statusText}`);
  }
  return res.blob();
}
