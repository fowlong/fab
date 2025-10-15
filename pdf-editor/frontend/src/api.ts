import type { DocumentIR, PatchOp, PatchResponse } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

async function json<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return response.json() as Promise<T>;
}

export async function openDocument(file: File): Promise<{ docId: string; ir: DocumentIR }> {
  const form = new FormData();
  form.append('file', file);
  const openResponse = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: form,
  });
  const { docId } = await json<{ docId: string }>(openResponse);
  const irResponse = await fetch(`${API_BASE}/api/ir/${docId}`);
  const ir = await json<DocumentIR>(irResponse);
  return { docId, ir };
}

export async function sendPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops),
  });
  return json<PatchResponse>(response);
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const response = await fetch(`${API_BASE}/api/pdf/${docId}`);
  if (!response.ok) {
    throw new Error('Unable to download PDF');
  }
  return response.blob();
}
