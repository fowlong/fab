import { Canvas, Rect } from 'fabric/es';
import type { Matrix, PageIR, PageObject } from './types';
import { S, fabricDeltaToPdfDelta, multiply, ptToPxMatrix } from './coords';

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
};

type ControllerState = {
  object: Rect;
  meta: ControllerMeta;
  baseMatrix: Matrix;
  pending: boolean;
};

type TransformArgs = ControllerMeta & {
  deltaMatrixPt: Matrix;
};

type TransformCallback = (args: TransformArgs) => Promise<void>;

export class FabricOverlay {
  private canvas: Canvas;
  private readonly pageHeightPt: number;
  private readonly transform: TransformCallback;

  constructor(
    host: HTMLElement,
    size: { width: number; height: number },
    pageHeightPt: number,
    onTransform: TransformCallback,
  ) {
    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.className = 'fabric-page-overlay';
    host.innerHTML = '';
    host.appendChild(canvasEl);

    this.canvas = new Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });
    this.pageHeightPt = pageHeightPt;
    this.transform = onTransform;
  }

  dispose() {
    this.canvas.dispose();
  }

  setPage(page: PageIR) {
    this.canvas.getObjects().forEach((obj) => this.canvas.remove(obj));

    page.objects.forEach((object) => {
      const controller = this.createController(page, object);
      this.canvas.add(controller.object);
      controller.object.on('modified', () => {
        void this.handleModified(controller);
      });
      controller.object.on('mousedown', () => {
        controller.object.set('strokeWidth', 2);
      });
      controller.object.on('mouseup', () => {
        controller.object.set('strokeWidth', 1.5);
      });
      controller.object.on('deselected', () => {
        controller.object.set('strokeWidth', 1.5);
      });
    });

    this.canvas.requestRenderAll();
  }

  private createController(page: PageIR, object: PageObject): ControllerState {
    const [x0, yMin, x1, yMax] = object.bbox;
    const widthPt = Math.max(x1 - x0, 4);
    const heightPt = Math.max(yMax - yMin, 4);
    const widthPx = widthPt / S;
    const heightPx = heightPt / S;

    const rect = new Rect({
      left: 0,
      top: 0,
      width: widthPx,
      height: heightPx,
      fill: 'rgba(37, 99, 235, 0.08)',
      stroke: 'rgba(37, 99, 235, 0.9)',
      strokeDashArray: [8, 4],
      strokeWidth: 1.5,
      hasBorders: true,
      hasControls: true,
      transparentCorners: false,
      objectCaching: false,
      selectable: true,
      cornerColor: '#1d4ed8',
      cornerStrokeColor: '#1d4ed8',
      borderColor: '#1d4ed8',
      originX: 'left',
      originY: 'top',
    });

    const matrixPt = object.kind === 'text' ? object.Tm : object.cm;
    const matrixPx = multiply(ptToPxMatrix(page.heightPt), matrixPt);
    this.applyMatrix(rect, matrixPx);
    rect.setCoords();

    const state: ControllerState = {
      object: rect,
      meta: {
        id: object.id,
        kind: object.kind,
        pageIndex: page.index,
      },
      baseMatrix: this.readMatrix(rect),
      pending: false,
    };

    return state;
  }

  private async handleModified(state: ControllerState) {
    if (state.pending) {
      this.applyMatrix(state.object, state.baseMatrix);
      state.object.setCoords();
      this.canvas.requestRenderAll();
      return;
    }

    const nextMatrix = this.readMatrix(state.object);
    const delta = fabricDeltaToPdfDelta(state.baseMatrix, nextMatrix, this.pageHeightPt);

    if (isIdentity(delta)) {
      state.object.setCoords();
      this.canvas.requestRenderAll();
      return;
    }

    state.pending = true;
    try {
      await this.transform({ ...state.meta, deltaMatrixPt: delta });
      state.baseMatrix = nextMatrix;
    } catch (err) {
      console.error('Failed to apply transform', err);
      this.applyMatrix(state.object, state.baseMatrix);
      state.object.setCoords();
    } finally {
      state.pending = false;
      this.canvas.requestRenderAll();
    }
  }

  private applyMatrix(target: Rect, matrix: Matrix) {
    const transform: number[] = [...matrix];
    target.set({
      left: 0,
      top: 0,
      transformMatrix: transform,
      hoverCursor: 'move',
    });
  }

  private readMatrix(target: Rect): Matrix {
    const matrix = target.calcTransformMatrix();
    if (!Array.isArray(matrix) || matrix.length < 6) {
      throw new Error('Invalid transform matrix');
    }
    return [matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5]];
  }
}

function isIdentity(matrix: Matrix): boolean {
  const [a, b, c, d, e, f] = matrix;
  return (
    Math.abs(a - 1) < 1e-6 &&
    Math.abs(b) < 1e-6 &&
    Math.abs(c) < 1e-6 &&
    Math.abs(d - 1) < 1e-6 &&
    Math.abs(e) < 1e-4 &&
    Math.abs(f) < 1e-4
  );
}
