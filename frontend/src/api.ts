import type { DocumentIR, Patch, PatchResponse } from './types';

const DEFAULT_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export type OpenResponse = {
  docId: string;
};

export async function open(file: File): Promise<OpenResponse> {
  const formData = new FormData();
  formData.set('file', file);
  const response = await fetch(`${DEFAULT_BASE}/api/open`, {
    method: 'POST',
    body: formData,
  });
  if (!response.ok) {
    throw new Error(`open failed: ${response.status}`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${DEFAULT_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`IR fetch failed: ${response.status}`);
  }
  return response.json();
}

export async function patch(docId: string, ops: Patch[]): Promise<PatchResponse> {
  const response = await fetch(`${DEFAULT_BASE}/api/patch/${encodeURIComponent(docId)}`, {
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

export async function download(docId: string): Promise<void> {
  const response = await fetch(`${DEFAULT_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`download failed: ${response.status}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `${docId}.pdf`;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}

export async function fetchPdf(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${DEFAULT_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`pdf fetch failed: ${response.status}`);
  }
  return response.arrayBuffer();
}
