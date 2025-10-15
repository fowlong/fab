import { fabric } from 'fabric';
import type { EditorContext, PdfPreview, IrObject } from './types';
import { fabricDeltaToPdfDelta } from './coords';
import { sendPatch } from './api';

export function initFabricOverlay(ctx: EditorContext, preview: PdfPreview) {
  preview.pages.forEach((page) => {
    const canvasEl = document.createElement('canvas');
    canvasEl.className = 'fabric-overlay';
    canvasEl.width = page.canvas.width;
    canvasEl.height = page.canvas.height;
    canvasEl.style.width = `${page.canvas.style.width || page.canvas.width + 'px'}`;
    canvasEl.style.height = `${page.canvas.style.height || page.canvas.height + 'px'}`;
    page.container.appendChild(canvasEl);

    const fabricCanvas = new fabric.Canvas(canvasEl, {
      selection: true,
      backgroundColor: 'rgba(0,0,0,0)'
    });

    ctx.overlayByPage.set(page.pageIndex, fabricCanvas);

    const irPage = ctx.pages.find((p) => p.index === page.pageIndex);
    if (!irPage) {
      return;
    }

    const scale = Number(page.canvas.dataset.scale ?? '1');
    const pageHeightPt = irPage.heightPt;

    irPage.objects.forEach((obj) => {
      const rect = createControllerForObject(obj, pageHeightPt, scale);
      if (!rect) return;
      rect.lockRotation = false;
      rect.lockScalingFlip = true;
      rect.lockSkewingX = true;
      rect.lockSkewingY = true;
      rect.set('hasRotatingPoint', true);
      rect.set('transparentCorners', false);
      rect.set('cornerColor', '#1976d2');
      rect.set('stroke', obj.kind === 'text' ? '#4caf50' : '#ff9800');
      rect.set('fill', 'rgba(0,0,0,0)');

      rect.on('modified', () => {
        if (!ctx.docId) return;
        const original = rect.data?.originalMatrix as number[] | undefined;
        if (!original) return;
        const current = rect.calcTransformMatrix();
        const pageHeightPt = irPage.heightPt;
        const delta = fabricDeltaToPdfDelta(
          original as unknown as [number, number, number, number, number, number],
          current as unknown as [number, number, number, number, number, number],
          pageHeightPt
        );
        void sendPatch(ctx.docId!, [
          {
            op: 'transform',
            target: { page: irPage.index, id: obj.id },
            deltaMatrixPt: delta,
            kind: obj.kind === 'path' ? 'path' : obj.kind
          }
        ]);
        rect.set('data', { ...rect.data, originalMatrix: current });
      });

      rect.set('data', {
        page: page.pageIndex,
        id: obj.id,
        originalMatrix: rect.calcTransformMatrix()
      });

      fabricCanvas.add(rect);
    });

    fabricCanvas.renderAll();
  });
}

function createControllerForObject(
  obj: IrObject,
  pageHeightPt: number,
  scale: number
): fabric.Rect | null {
  const [x1, y1, x2, y2] = obj.bbox;
  if (isNaN(x1) || isNaN(y1) || isNaN(x2) || isNaN(y2)) {
    return null;
  }

  const ptToPx = 96 / 72;
  const widthPx = (x2 - x1) * ptToPx * scale;
  const heightPx = (y2 - y1) * ptToPx * scale;
  const left = x1 * ptToPx * scale;
  const top = (pageHeightPt - y2) * ptToPx * scale;

  const rect = new fabric.Rect({
    left,
    top,
    width: widthPx,
    height: heightPx,
    strokeWidth: 1,
    fill: 'rgba(0,0,0,0)',
    stroke: '#1976d2'
  });

  return rect;
}
