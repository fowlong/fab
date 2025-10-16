import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const DEFAULT_BASE =
  ((typeof import.meta !== 'undefined' ? (import.meta as any).env?.VITE_API_BASE : undefined) as
    | string
    | undefined) ?? (globalThis as any).__API_BASE ?? 'http://localhost:8787';

export type OpenResponse = {
  docId: string;
};

export async function openDocument(file: File): Promise<OpenResponse> {
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

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${DEFAULT_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`IR fetch failed: ${response.status}`);
  }
  return response.json();
}

export async function postPatch(
  docId: string,
  ops: PatchOperation[],
): Promise<PatchResponse> {
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

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${DEFAULT_BASE}/api/pdf/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`download failed: ${response.status}`);
  }
  return response.blob();
}
