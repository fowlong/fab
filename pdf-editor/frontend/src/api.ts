import type { DocumentIr, OpenResponse, PatchOp, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

async function request<T>(input: RequestInfo, init?: RequestInit): Promise<T> {
  const response = await fetch(input, init);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Request failed (${response.status}): ${text}`);
  }
  return (await response.json()) as T;
}

export async function openDocument(base64Pdf: string, filename?: string): Promise<OpenResponse> {
  return request<OpenResponse>(`${API_BASE}/api/open`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ data: base64Pdf, filename })
  });
}

export async function fetchIr(docId: string): Promise<DocumentIr> {
  return request<DocumentIr>(`${API_BASE}/api/ir/${docId}`);
}

export async function applyPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  return request<PatchResponse>(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops)
  });
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!response.ok) {
    throw new Error(`Failed to download PDF (${response.status})`);
  }
  return await response.blob();
}
