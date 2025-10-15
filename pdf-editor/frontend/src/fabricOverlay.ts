import { fabric } from 'fabric';
import type { Canvas as FabricCanvas, FabricObject } from 'fabric';
import type { DocumentIR, PageObject, PatchOp } from './types';
import { createController, fabricMatrixFromObject, type FabricBinding } from './mapping';
import { fabricDeltaToPdfDelta } from './coords';

export type OverlayContext = {
  canvas: FabricCanvas;
  bindings: Map<string, FabricBinding>;
  pageHeightPt: number;
  docId: string;
  pageIndex: number;
  onPatch: (ops: PatchOp[]) => Promise<void>;
};

export function initFabricCanvas(canvasEl: HTMLCanvasElement): FabricCanvas {
  const canvas = new fabric.Canvas(canvasEl, {
    selection: true,
    preserveObjectStacking: true
  });
  return canvas;
}

export function buildOverlay(
  ctx: OverlayContext,
  pageObjects: PageObject[]
) {
  ctx.canvas.clear();
  ctx.bindings.clear();
  pageObjects.forEach((obj) => {
    const binding = createController(obj, ctx.pageHeightPt);
    ctx.canvas.add(binding.fabricObject);
    ctx.bindings.set(obj.id, binding);
    binding.fabricObject.on('modified', () => handleModified(ctx, binding.fabricObject, obj));
  });
}

async function handleModified(
  ctx: OverlayContext,
  fabricObject: FabricObject,
  obj: PageObject
) {
  const binding = ctx.bindings.get(obj.id);
  if (!binding) {
    return;
  }
  const oldMatrix = binding.baseMatrixPx;
  const newMatrix = fabricMatrixFromObject(fabricObject);
  const delta = fabricDeltaToPdfDelta(oldMatrix, newMatrix, ctx.pageHeightPt);
  const op: PatchOp = {
    op: 'transform',
    target: { page: ctx.pageIndex, id: obj.id },
    deltaMatrixPt: delta,
    kind: obj.kind
  } as PatchOp;
  await ctx.onPatch([op]);
  binding.baseMatrixPx = newMatrix;
}

export function wireTextEditing(
  ctx: OverlayContext,
  ir: DocumentIR,
  container: HTMLElement
) {
  ctx.canvas.on('mouse:dblclick', (evt) => {
    const target = evt.target as FabricObject | undefined;
    if (!target) {
      return;
    }
    const bindingEntry = Array.from(ctx.bindings.values()).find(
      (b) => b.fabricObject === target
    );
    if (!bindingEntry || bindingEntry.irObject.kind !== 'text') {
      return;
    }
    openTextEditor(ctx, bindingEntry, container);
  });
}

function openTextEditor(ctx: OverlayContext, binding: FabricBinding, container: HTMLElement) {
  const editor = document.createElement('input');
  editor.className = 'text-editor';
  editor.value = binding.irObject.kind === 'text' ? binding.irObject.unicode : '';
  const rect = binding.fabricObject.getBoundingRect();
  editor.style.left = `${rect.left}px`;
  editor.style.top = `${rect.top}px`;
  editor.style.width = `${rect.width}px`;
  container.appendChild(editor);
  editor.focus();

  const commit = async () => {
    if (binding.irObject.kind !== 'text') {
      return;
    }
    const newText = editor.value;
    container.removeChild(editor);
    if (newText === binding.irObject.unicode) {
      return;
    }
    const op: PatchOp = {
      op: 'editText',
      target: { page: ctx.pageIndex, id: binding.irObject.id },
      text: newText,
      fontPref: { preferExisting: true, fallbackFamily: 'Noto Sans' }
    };
    await ctx.onPatch([op]);
  };

  editor.addEventListener('blur', commit, { once: true });
  editor.addEventListener('keydown', (evt) => {
    if (evt.key === 'Enter') {
      commit();
    }
    if (evt.key === 'Escape') {
      container.removeChild(editor);
    }
  });
}
