// @ts-nocheck
import assert from 'node:assert/strict';
import test from 'node:test';
import { installDom, TestElement } from './dom-stub';
import { PdfPreview } from '../src/pdfPreview';

test('PdfPreview renders canvas elements and tracks sizes', async (t) => {
  const restoreDom = installDom();
  t.after(() => restoreDom());

  const container = new TestElement('div');
  (globalThis as any).document.body.appendChild(container);
  const preview = new PdfPreview(container as unknown as HTMLElement);
  const buffer = new ArrayBuffer(8);

  await preview.load(buffer);

  assert.equal(container.children.length, 1);
  assert.deepEqual(preview.getSizes(), [{ width: 200, height: 300 }]);
});

test('PdfPreview reset clears canvases and sizes', (t) => {
  const restoreDom = installDom();
  t.after(() => restoreDom());

  const container = new TestElement('div');
  (globalThis as any).document.body.appendChild(container);
  const preview = new PdfPreview(container as unknown as HTMLElement);
  container.appendChild(new TestElement('canvas'));

  preview.reset();

  assert.equal(container.children.length, 0);
  assert.deepEqual(preview.getSizes(), []);
});
