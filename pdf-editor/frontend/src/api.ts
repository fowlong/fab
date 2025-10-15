import type { DocumentIR, PatchOp, PatchResponse } from './types';

declare const __API_BASE__: string;

const API_BASE = __API_BASE__;

export async function openDocument(file: File): Promise<{ docId: string; ir: DocumentIR }> {
  const form = new FormData();
  form.append('file', file, file.name);
  const res = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: form,
  });
  if (!res.ok) {
    throw new Error(`Failed to open PDF (${res.status})`);
  }
  const { docId } = await res.json();
  const ir = await fetchIR(docId);
  return { docId, ir };
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const res = await fetch(`${API_BASE}/api/ir/${docId}`);
  if (!res.ok) {
    throw new Error(`Failed to fetch IR (${res.status})`);
  }
  return res.json();
}

export async function postPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const res = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops),
  });
  if (!res.ok) {
    throw new Error(`Failed to apply patch (${res.status})`);
  }
  return res.json();
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!res.ok) {
    throw new Error(`Failed to download PDF (${res.status})`);
  }
  return res.blob();
}
