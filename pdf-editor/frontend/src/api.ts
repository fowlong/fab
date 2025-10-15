import type { DocumentIR, PatchOperation, PatchResponse } from './types';

const DEFAULT_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';

export class ApiClient {
  private readonly baseUrl: string;

  constructor(baseUrl: string = DEFAULT_BASE) {
    this.baseUrl = baseUrl;
  }

  async open(file: File): Promise<{ docId: string }> {
    const form = new FormData();
    form.append('file', file);

    const response = await fetch(`${this.baseUrl}/api/open`, {
      method: 'POST',
      body: form,
    });

    if (!response.ok) {
      throw new Error('Failed to open PDF');
    }

    return response.json();
  }

  async fetchIR(docId: string): Promise<DocumentIR> {
    const response = await fetch(`${this.baseUrl}/api/ir/${docId}`);
    if (!response.ok) {
      throw new Error('Failed to fetch IR');
    }
    return response.json();
  }

  async patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
    const response = await fetch(`${this.baseUrl}/api/patch/${docId}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(ops),
    });

    if (!response.ok) {
      throw new Error('Patch request failed');
    }

    return response.json();
  }

  async download(docId: string): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/api/pdf/${docId}`);
    if (!response.ok) {
      throw new Error('Failed to download PDF');
    }

    return response.blob();
  }
}
