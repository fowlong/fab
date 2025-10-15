import type { IrDocument, PatchOperation, PatchResponse } from './types';

const API_BASE = (window as any).__API_BASE__ ?? 'http://localhost:8787';

interface OpenResponse {
  docId: string;
}

export async function loadInitialDocument(file: File): Promise<{ docId: string; ir: IrDocument }> {
  const formData = new FormData();
  formData.append('file', file);

  const openResp = await fetch(`${API_BASE}/api/open`, {
    method: 'POST',
    body: formData
  });
  if (!openResp.ok) {
    throw new Error('Failed to open PDF');
  }
  const openJson = (await openResp.json()) as OpenResponse;

  const irResp = await fetch(`${API_BASE}/api/ir/${openJson.docId}`);
  if (!irResp.ok) {
    throw new Error('Failed to fetch IR');
  }
  const ir = (await irResp.json()) as IrDocument;
  return { docId: openJson.docId, ir };
}

export async function sendPatch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
  const response = await fetch(`${API_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops)
  });
  if (!response.ok) {
    return { ok: false, error: 'Failed to apply patch' };
  }
  return (await response.json()) as PatchResponse;
}
