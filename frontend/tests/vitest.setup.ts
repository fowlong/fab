import { afterEach, beforeEach } from 'vitest';
import { createdCanvases } from './mocks/fabric';

beforeEach(() => {
  createdCanvases.length = 0;
});

afterEach(() => {
  createdCanvases.length = 0;
  if (typeof globalThis.document !== 'undefined' && 'body' in globalThis.document) {
    const body = (globalThis as any).document.body;
    if (body && typeof body.children !== 'undefined') {
      body.children = [];
    }
  }
});
