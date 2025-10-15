import { fabric } from 'fabric';
import { fabricDeltaToPdfDelta } from './coords';
import { createController, type FabricController } from './mapping';
import type { Matrix, PageIR } from './types';

export interface OverlayCallbacks {
  onTransform: (
    targetId: string,
    pageIndex: number,
    deltaMatrixPt: Matrix,
    kind: PageIR['objects'][number]['kind']
  ) => Promise<void> | void;
}

interface OverlayPage {
  canvas: fabric.Canvas;
  canvasElement: HTMLCanvasElement;
  controllers: Map<string, FabricController>;
  heightPt: number;
}

export class FabricOverlayManager {
  private root: HTMLElement;
  private callbacks: OverlayCallbacks;
  private pages: OverlayPage[] = [];

  constructor(root: HTMLElement, callbacks: OverlayCallbacks) {
    this.root = root;
    this.callbacks = callbacks;
  }

  reset(): void {
    this.pages.forEach((page) => page.canvas.dispose());
    this.pages.forEach((page) => page.canvasElement.remove());
    this.pages = [];
  }

  mountPages(pages: PageIR[], containers: HTMLElement[]): void {
    this.reset();
    pages.forEach((page) => {
      const container = containers[page.index];
      if (!container) {
        throw new Error(`Missing container for page ${page.index}`);
      }
      const canvasEl = document.createElement('canvas');
      canvasEl.id = `fabric-p${page.index}`;
      canvasEl.width = Math.round((page.widthPt / 72) * 96);
      canvasEl.height = Math.round((page.heightPt / 72) * 96);
      canvasEl.className = 'fabric-overlay';
      container.appendChild(canvasEl);
      const canvas = new fabric.Canvas(canvasEl, {
        selection: false,
        uniformScaling: true,
        preserveObjectStacking: true
      });
      const overlayPage: OverlayPage = {
        canvas,
        canvasElement: canvasEl,
        controllers: new Map(),
        heightPt: page.heightPt
      };
      this.pages.push(overlayPage);
      page.objects.forEach((obj) => {
        const controller = createController(page.index, page.heightPt, obj);
        overlayPage.controllers.set(obj.id, controller);
        controller.object.data = {
          id: obj.id,
          kind: obj.kind,
          lastMatrix: controller.object.transformMatrix
        };
        canvas.add(controller.object);
      });
      canvas.on('object:modified', (evt) => {
        const target = evt.target as fabric.Object & { data?: any };
        if (!target?.data) return;
        const controller = overlayPage.controllers.get(target.data.id);
        if (!controller) return;
        const newMatrix = toMatrix(target.calcTransformMatrix());
        const previousMatrix: Matrix = target.data.lastMatrix ?? toMatrix(controller.object.calcTransformMatrix());
        const delta = fabricDeltaToPdfDelta(previousMatrix, newMatrix, overlayPage.heightPt);
        target.data.lastMatrix = newMatrix;
        this.callbacks.onTransform(target.data.id, page.index, delta, target.data.kind);
      });
    });
  }
}

function toMatrix(values: number[] | undefined): Matrix {
  if (!values || values.length < 6) {
    return [1, 0, 0, 1, 0, 0];
  }
  return [values[0], values[1], values[2], values[3], values[4], values[5]];
}
