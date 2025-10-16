import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export async function open(file: File): Promise<{ docId: string }> {
  const formData = new FormData();
  formData.set('file', file);
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: formData,
  });
  if (!response.ok) {
    throw new Error(`open failed: ${response.status}`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`ir fetch failed: ${response.status}`);
  }
  return response.json();
}

export async function patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error(`patch failed: ${response.status}`);
  }
  return response.json();
}

export async function download(docId: string): Promise<Blob> {
  const response = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`download failed: ${response.status}`);
  }
  return response.blob();
}
