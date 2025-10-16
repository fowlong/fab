import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export type OpenResponse = {
  docId: string;
};

export async function open(file: File): Promise<OpenResponse> {
  const formData = new FormData();
  formData.set('file', file);
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: formData,
  });
  if (!response.ok) {
    throw new Error(`Failed to open document (${response.status})`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch IR (${response.status})`);
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
    throw new Error(`Patch failed (${response.status})`);
  }
  return response.json();
}

export async function fetchPdfBytes(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${API_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch PDF (${response.status})`);
  }
  return response.arrayBuffer();
}

export async function download(docId: string): Promise<void> {
  const buffer = await fetchPdfBytes(docId);
  const blob = new Blob([buffer], { type: 'application/pdf' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${docId}.pdf`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
