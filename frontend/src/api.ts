import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = 'http://localhost:8787/api';

async function handleJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `Request failed with status ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function open(file: File): Promise<{ docId: string }> {
  const form = new FormData();
  form.append('file', file);
  const response = await fetch(`${API_BASE}/open`, {
    method: 'POST',
    body: form,
  });
  return handleJson<{ docId: string }>(response);
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/ir/${encodeURIComponent(docId)}`);
  return handleJson<DocumentIR>(response);
}

export async function fetchPdf(docId: string): Promise<ArrayBuffer> {
  const response = await fetch(`${API_BASE}/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `Failed to fetch PDF: ${response.status}`);
  }
  return response.arrayBuffer();
}

export async function patch(
  docId: string,
  ops: PatchOperation[],
): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(ops),
  });
  return handleJson<PatchResponse>(response);
}

export function download(docId: string): void {
  const link = document.createElement('a');
  link.href = `${API_BASE}/pdf/${encodeURIComponent(docId)}`;
  link.download = `${docId}.pdf`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
