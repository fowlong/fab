import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const BASE_URL = 'http://localhost:8787/api';

export async function open(file: File): Promise<{ docId: string }> {
  const form = new FormData();
  form.append('file', file);
  const response = await fetch(`${BASE_URL}/open`, {
    method: 'POST',
    body: form,
  });
  if (!response.ok) {
    throw new Error(`Failed to open document: ${response.statusText}`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${BASE_URL}/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch IR: ${response.statusText}`);
  }
  return response.json();
}

export async function getPdfBytes(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${BASE_URL.replace('/api', '')}/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Failed to fetch PDF bytes: ${response.statusText}`);
  }
  return response.arrayBuffer();
}

export async function patch(
  docId: string,
  operations: PatchOperation[],
): Promise<PatchResponse> {
  const response = await fetch(`${BASE_URL}/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(operations),
  });
  if (!response.ok) {
    throw new Error(`Failed to apply patch: ${response.statusText}`);
  }
  return response.json();
}

export async function download(docId: string): Promise<void> {
  const buffer = await getPdfBytes(docId);
  const blob = new Blob([buffer], { type: 'application/pdf' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${docId}.pdf`;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}
