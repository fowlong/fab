import type { DocumentIR, PatchOperation } from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? '/api';

async function postJson<T>(url: string, body: unknown): Promise<T> {
  const resp = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`Request failed: ${resp.status}`);
  return resp.json() as Promise<T>;
}

export async function loadDocumentIR(file: File): Promise<{ docId: string; ir: DocumentIR }> {
  const form = new FormData();
  form.set('file', file);
  const openResp = await fetch(`${API_BASE}/open`, {
    method: 'POST',
    body: form,
  });
  if (!openResp.ok) throw new Error('Failed to open PDF');
  const { docId } = await openResp.json();
  const irResp = await fetch(`${API_BASE}/ir/${docId}`);
  if (!irResp.ok) throw new Error('Failed to load IR');
  const ir = (await irResp.json()) as DocumentIR;
  return { docId, ir };
}

export async function sendPatch(docId: string, ops: PatchOperation[]) {
  return postJson(`${API_BASE}/patch/${docId}`, ops);
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const resp = await fetch(`${API_BASE}/pdf/${docId}`);
  if (!resp.ok) throw new Error('Failed to download PDF');
  return resp.blob();
}
