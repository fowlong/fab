import { afterEach, describe, expect, it } from 'vitest';
import { installDom, TestElement } from './dom-stub';
import { PdfPreview } from '../src/pdfPreview';

describe('PdfPreview', () => {
  let restoreDom: (() => void) | undefined;

  afterEach(() => {
    restoreDom?.();
    restoreDom = undefined;
  });

  it('renders canvas elements and tracks sizes', async () => {
    restoreDom = installDom();
    const container = new TestElement('div');
    (globalThis as any).document.body.appendChild(container);
    const preview = new PdfPreview(container as unknown as HTMLElement);
    const buffer = new ArrayBuffer(8);

    await preview.load(buffer);

    expect(container.children.length).toBe(1);
    expect(preview.getSizes()).toEqual([{ width: 200, height: 300 }]);
  });

  it('reset clears canvases and sizes', () => {
    restoreDom = installDom();
    const container = new TestElement('div');
    (globalThis as any).document.body.appendChild(container);
    const preview = new PdfPreview(container as unknown as HTMLElement);
    container.appendChild(new TestElement('canvas'));

    preview.reset();

    expect(container.children.length).toBe(0);
    expect(preview.getSizes()).toEqual([]);
  });
});
