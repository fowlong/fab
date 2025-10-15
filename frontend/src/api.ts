import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const BASE_URL = 'http://localhost:8787';

export async function open(file: File): Promise<{ docId: string }> {
  const form = new FormData();
  form.append('file', file);
  const response = await fetch(`${BASE_URL}/api/open`, {
    method: 'POST',
    body: form,
  });
  if (!response.ok) {
    throw new Error(`Failed to open document: ${response.status}`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${BASE_URL}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch IR: ${response.status}`);
  }
  return response.json();
}

export async function fetchPdf(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${BASE_URL}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch PDF: ${response.status}`);
  }
  return response.arrayBuffer();
}

export async function patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  const response = await fetch(`${BASE_URL}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error(`Patch request failed: ${response.status}`);
  }
  return response.json();
}

export async function download(docId: string) {
  const response = await fetch(`${BASE_URL}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to download PDF: ${response.status}`);
  }
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `${docId}.pdf`;
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
