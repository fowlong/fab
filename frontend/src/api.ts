import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export async function open(file: File): Promise<{ docId: string }> {
  const formData = new FormData();
  formData.set('file', file);
  const response = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: formData,
  });
  if (!response.ok) {
    throw new Error(`Open request failed with ${response.status}`);
  }
  return (await response.json()) as { docId: string };
}

export async function getIR(docId: string): Promise<DocumentIR> {
  const response = await fetch(`${API_BASE}/api/ir/${encodeURIComponent(docId)}`);
  if (!response.ok) {
    throw new Error(`IR request failed with ${response.status}`);
  }
  return (await response.json()) as DocumentIR;
}

export async function patch(
  docId: string,
  ops: PatchOperation[],
): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${encodeURIComponent(docId)}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(ops),
  });
  if (!response.ok) {
    throw new Error(`Patch request failed with ${response.status}`);
  }
  return (await response.json()) as PatchResponse;
}

export function download(docId: string): void {
  const link = document.createElement('a');
  link.href = `${API_BASE}/api/pdf/${encodeURIComponent(docId)}`;
  link.rel = 'noopener';
  link.download = `${docId}.pdf`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
