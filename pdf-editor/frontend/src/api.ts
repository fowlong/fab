import type {
  DocumentId,
  DocumentIr,
  OpenRequest,
  OpenResponse,
  PatchOp,
  PatchResponse,
} from './types';
import { __API_BASE__ } from './types';

export class ApiClient {
  constructor(private readonly baseUrl: string = __API_BASE__) {}

  async open(file: File): Promise<DocumentId> {
    const buffer = await file.arrayBuffer();
    const base64 = arrayBufferToBase64(buffer);
    const payload: OpenRequest = { pdf_base64: base64 };
    const response = await fetch(new URL('/api/open', this.baseUrl), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if (!response.ok) {
      throw new Error(`Failed to open PDF: ${response.statusText}`);
    }
    const json = (await response.json()) as OpenResponse;
    return json.doc_id;
  }

  async fetchIr(docId: DocumentId): Promise<DocumentIr> {
    const response = await fetch(new URL(`/api/ir/${docId}`, this.baseUrl));
    if (!response.ok) {
      throw new Error('Failed to fetch IR');
    }
    return (await response.json()) as DocumentIr;
  }

  async applyPatch(docId: DocumentId, ops: PatchOp[]): Promise<PatchResponse> {
    const response = await fetch(new URL(`/api/patch/${docId}`, this.baseUrl), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ops }),
    });
    if (!response.ok) {
      throw new Error('Patch request failed');
    }
    return (await response.json()) as PatchResponse;
  }

  async download(docId: DocumentId): Promise<Blob> {
    const response = await fetch(new URL(`/api/pdf/${docId}`, this.baseUrl));
    if (!response.ok) {
      throw new Error('Failed to download PDF');
    }
    return await response.blob();
  }
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  let binary = '';
  const bytes = new Uint8Array(buffer);
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    const sub = bytes.subarray(i, i + chunk);
    binary += String.fromCharCode(...sub);
  }
  return btoa(binary);
}

