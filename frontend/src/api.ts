import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const BASE_URL = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

type OpenResponse = {
  docId: string;
};

export async function open(file: File): Promise<OpenResponse> {
  const form = new FormData();
  form.set('file', file);
  const response = await fetch(`${BASE_URL}/api/open`, {
    method: 'POST',
    body: form,
  });
  if (!response.ok) {
    throw new Error(`Failed to open PDF (status ${response.status})`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${BASE_URL}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch IR (status ${response.status})`);
  }
  return response.json();
}

export async function patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  const response = await fetch(`${BASE_URL}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error(`Failed to apply patch (status ${response.status})`);
  }
  return response.json();
}

export async function fetchPdfBytes(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${BASE_URL}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch PDF (status ${response.status})`);
  }
  return response.arrayBuffer();
}

export async function download(docId: string): Promise<void> {
  const buffer = await fetchPdfBytes(docId);
  const blob = new Blob([buffer], { type: 'application/pdf' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `${docId}.pdf`;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}
