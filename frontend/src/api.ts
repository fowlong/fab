import type { DocumentIR, OpenResponse, PatchOp, PatchResponse } from './types';

const API_BASE: string = (globalThis as any).__API_BASE__ ?? 'http://localhost:8787';

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Request failed: ${response.status} ${text}`);
  }
  return response.json() as Promise<T>;
}

export async function openDocument(file: File): Promise<OpenResponse> {
  const form = new FormData();
  form.append('file', file);
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: form
  });
  return handleResponse<OpenResponse>(response);
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  return handleResponse<DocumentIR>(response);
}

export async function sendPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops)
  });
  return handleResponse<PatchResponse>(response);
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Download failed: ${response.status}`);
  }
  return response.blob();
}
