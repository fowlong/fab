import type { DocumentIR, PatchOp, PatchResponse } from './types';

const DEFAULT_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export async function openDocument(file: File): Promise<{ docId: string } & DocumentIR> {
  const form = new FormData();
  form.append('file', file);
  const openResp = await fetch(`${DEFAULT_BASE}/api/open`, {
    method: 'POST',
    body: form,
  });
  if (!openResp.ok) {
    throw new Error(`Failed to open PDF: ${openResp.statusText}`);
  }
  const { docId } = (await openResp.json()) as { docId: string };
  const irResp = await fetch(`${DEFAULT_BASE}/api/ir/${docId}`);
  if (!irResp.ok) {
    throw new Error(`Failed to load IR: ${irResp.statusText}`);
  }
  const ir = (await irResp.json()) as DocumentIR;
  return { docId, ...ir };
}

export async function fetchIR(docId: string): Promise<DocumentIR> {
  const resp = await fetch(`${DEFAULT_BASE}/api/ir/${docId}`);
  if (!resp.ok) {
    throw new Error(`Failed to fetch IR: ${resp.statusText}`);
  }
  return (await resp.json()) as DocumentIR;
}

export async function sendPatch(docId: string, ops: PatchOp[]): Promise<PatchResponse> {
  const resp = await fetch(`${DEFAULT_BASE}/api/patch/${docId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(ops),
  });
  if (!resp.ok) {
    return {
      ok: false,
      error: await resp.text(),
    };
  }
  return (await resp.json()) as PatchResponse;
}

export async function downloadPdf(docId: string): Promise<Blob> {
  const resp = await fetch(`${DEFAULT_BASE}/api/pdf/${docId}`);
  if (!resp.ok) {
    throw new Error(`Failed to download PDF: ${resp.statusText}`);
  }
  return await resp.blob();
}
