import type { DocumentIR, PatchOperation, PatchResponse } from './types';

export interface ApiClientOptions {
  baseUrl?: string;
}

export class ApiClient {
  private readonly baseUrl: string;

  constructor(options: ApiClientOptions = {}) {
    this.baseUrl = options.baseUrl ?? (import.meta.env.VITE_API_BASE ?? '/api');
  }

  async open(file: File): Promise<{ docId: string }> {
    const form = new FormData();
    form.append('file', file);
    const res = await fetch(`${this.baseUrl}/open`, {
      method: 'POST',
      body: form
    });
    if (!res.ok) {
      throw new Error(`Failed to open PDF: ${res.statusText}`);
    }
    return res.json();
  }

  async getIR(docId: string): Promise<DocumentIR> {
    const res = await fetch(`${this.baseUrl}/ir/${encodeURIComponent(docId)}`);
    if (!res.ok) {
      throw new Error(`Failed to fetch IR: ${res.statusText}`);
    }
    return res.json();
  }

  async patch(docId: string, ops: PatchOperation[]): Promise<PatchResponse> {
    const res = await fetch(`${this.baseUrl}/patch/${encodeURIComponent(docId)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(ops)
    });
    if (!res.ok) {
      throw new Error(`Failed to apply patch: ${res.statusText}`);
    }
    return res.json();
  }

  async download(docId: string): Promise<Blob> {
    const res = await fetch(`${this.baseUrl}/pdf/${encodeURIComponent(docId)}`);
    if (!res.ok) {
      throw new Error(`Failed to download PDF: ${res.statusText}`);
    }
    return res.blob();
  }
}
