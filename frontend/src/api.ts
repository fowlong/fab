import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const BASE_URL = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export type OpenResponse = {
  docId: string;
};

export async function open(file: File): Promise<OpenResponse> {
  const body = new FormData();
  body.set('file', file);
  const response = await fetch(`${BASE_URL}/api/open`, {
    method: 'POST',
    body,
  });
  if (!response.ok) {
    throw new Error(`Open failed with status ${response.status}`);
  }
  return response.json();
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${BASE_URL}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`IR request failed with status ${response.status}`);
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
    throw new Error(`Patch failed with status ${response.status}`);
  }
  return response.json();
}

export async function download(docId: string): Promise<Blob> {
  const response = await fetch(`${BASE_URL}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`Download failed with status ${response.status}`);
  }
  return response.blob();
}
