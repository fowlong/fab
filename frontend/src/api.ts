import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export async function open(file?: File): Promise<{ docId: string }> {
  const formData = new FormData();
  if (file) {
    formData.set('file', file);
  }
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: formData,
  });
  if (!response.ok) {
    throw new Error(`open failed: ${response.status}`);
  }
  return response.json() as Promise<{ docId: string }>;
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`IR fetch failed: ${response.status}`);
  }
  return response.json() as Promise<DocumentIR>;
}

export async function patch(
  docId: string,
  ops: PatchOperation[],
): Promise<PatchResponse> {
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
  return response.json() as Promise<PatchResponse>;
}

export async function fetchPdf(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`pdf fetch failed: ${response.status}`);
  }
  return response.arrayBuffer();
}

export async function download(docId: string): Promise<void> {
  const response = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`download failed: ${response.status}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `${docId}.pdf`;
  anchor.click();
  URL.revokeObjectURL(url);
}
