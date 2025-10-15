import { fabric } from 'fabric';
import { fabricDeltaToPdfDelta, multiplyMatrix } from './coords';
import { getMeta, pageObjectToFabric, updateBaseMatrix, fabricMatrix } from './mapping';
import type {
  DocumentIR,
  PageIR,
  PatchOp,
  PatchResponse,
  PatchTarget,
  PageObject,
} from './types';
import { postPatch } from './api';

export interface OverlayContext {
  docId: string;
  ir: DocumentIR;
  pdfData: ArrayBuffer;
  canvasByPage: Map<number, fabric.Canvas>;
  dispatchToast(message: string, tone?: 'info' | 'success' | 'error'): void;
  onPdfUpdated?(response: PatchResponse): void;
}

export type ModificationHandler = (target: PatchTarget, delta: number[]) => PatchOp;

export function createOverlayCanvas(container: HTMLElement, width: number, height: number) {
  const canvasEl = document.createElement('canvas');
  canvasEl.width = width;
  canvasEl.height = height;
  canvasEl.className = 'fabric-overlay';
  container.appendChild(canvasEl);
  const canvas = new fabric.Canvas(canvasEl, {
    selection: false,
    preserveObjectStacking: true,
    stopContextMenu: true,
  });
  return canvas;
}

export function populateOverlay(canvas: fabric.Canvas, page: PageIR) {
  canvas.clear();
  page.objects.forEach((obj) => {
    const fabricObj = pageObjectToFabric(canvas, page, obj, fabric);
    fabricObj.set('hasBorders', true);
    fabricObj.set('hasControls', true);
  });
}

export function installInteractionHandlers(context: OverlayContext) {
  context.canvasByPage.forEach((canvas, pageIndex) => {
    canvas.on('object:modified', async (event) => {
      const target = event.target;
      if (!target) return;
      const meta = getMeta(target);
      if (!meta) return;
      const page = context.ir.pages.find((p) => p.index === pageIndex);
      if (!page) return;
      const base = meta.baseMatrixPx;
      const current = fabricMatrix(target);
      const delta = fabricDeltaToPdfDelta(base, current, page.heightPt);
      const patch: PatchOp = {
        op: 'transform',
        target: { page: pageIndex, id: meta.id },
        deltaMatrixPt: delta,
        kind: findObjectKind(page, meta.id),
      };
      try {
        const response = await postPatch(context.docId, [patch]);
        updateBaseMatrix(target, current);
        context.dispatchToast('Transform saved', 'success');
        context.onPdfUpdated?.(response);
      } catch (err) {
        context.dispatchToast((err as Error).message, 'error');
      }
    });
  });
}

function findObjectKind(page: PageIR, id: string): 'text' | 'image' | 'path' {
  const obj = page.objects.find((o) => o.id === id);
  if (!obj) {
    throw new Error(`Unable to resolve object ${id}`);
  }
  return obj.kind;
}

export function refreshFromIr(context: OverlayContext, newIr: DocumentIR) {
  context.ir = newIr;
  newIr.pages.forEach((page) => {
    let canvas = context.canvasByPage.get(page.index);
    if (!canvas) {
      throw new Error(`Canvas for page ${page.index} not initialised`);
    }
    populateOverlay(canvas, page);
  });
}

export function applyTransformToOverlay(
  context: OverlayContext,
  pageIndex: number,
  objectId: string,
  delta: [number, number, number, number, number, number],
) {
  const page = context.ir.pages.find((p) => p.index === pageIndex);
  if (!page) return;
  const canvas = context.canvasByPage.get(pageIndex);
  if (!canvas) return;
  const fabricObj = canvas.getObjects().find((o) => getMeta(o)?.id === objectId);
  if (!fabricObj) return;
  const current = fabricMatrix(fabricObj);
  const next = multiplyMatrix(current, delta);
  fabricObj.set({ transformMatrix: next });
  updateBaseMatrix(fabricObj, next);
  canvas.requestRenderAll();
}

export function createInlineTextEditor(
  canvas: fabric.Canvas,
  object: PageObject,
  page: PageIR,
  onCommit: (value: string) => void,
) {
  const meta = getMeta(object as unknown as fabric.Object);
  if (!meta) return;
  const bounds = canvas.getSelectionElement().getBoundingClientRect();
  const input = document.createElement('input');
  input.type = 'text';
  input.value = 'text' in object ? object.unicode : '';
  input.className = 'inline-text-editor';
  Object.assign(input.style, {
    position: 'absolute',
    top: `${bounds.top}px`,
    left: `${bounds.left}px`,
    width: `${bounds.width}px`,
    transform: 'translate(-50%, -50%)',
  });
  const detach = () => {
    input.remove();
    window.removeEventListener('keydown', onKeyDown);
  };
  const onKeyDown = (ev: KeyboardEvent) => {
    if (ev.key === 'Enter') {
      onCommit(input.value);
      detach();
    } else if (ev.key === 'Escape') {
      detach();
    }
  };
  window.addEventListener('keydown', onKeyDown);
  document.body.appendChild(input);
  input.focus();
}
