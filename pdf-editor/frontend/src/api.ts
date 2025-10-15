import type { DocumentIR, PatchOp, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export interface OpenResponse {
  docId: string;
  ir: DocumentIR;
}

export async function loadSampleDocument(): Promise<OpenResponse | null> {
  try {
    const res = await fetch(`${API_BASE}/api/open-sample`);
    if (!res.ok) {
      return null;
    }
    const data = (await res.json()) as OpenResponse;
    return data;
  } catch (error) {
    console.warn('Failed to load sample document', error);
    return null;
  }
}

export async function openFile(file: File): Promise<OpenResponse> {
  const form = new FormData();
  form.append('file', file);
  const res = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: form,
  });
  if (!res.ok) {
    throw new Error('Failed to open file');
  }
  return (await res.json()) as OpenResponse;
}

export async function sendPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const res = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops),
  });
  if (!res.ok) {
    throw new Error(`Patch failed with status ${res.status}`);
  }
  return (await res.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error('Download failed');
  }
  return await res.blob();
}
