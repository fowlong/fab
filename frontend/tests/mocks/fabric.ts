// @ts-nocheck
class MockFabricObject {
  public options: Record<string, unknown>;
  public metadata: Record<string, unknown> = {};

  constructor(options: Record<string, unknown>) {
    this.options = options;
  }

  set(key: string, value: unknown) {
    this.metadata[key] = value;
  }
}

class MockRect extends MockFabricObject {}

class MockCanvas {
  public element: any;
  public options: Record<string, unknown>;
  public objects: any[] = [];
  public renderCount = 0;
  public disposed = false;

  constructor(element: any, options: Record<string, unknown>) {
    this.element = element;
    this.options = options;
    createdCanvases.push(this);
  }

  add(obj: any) {
    this.objects.push(obj);
  }

  renderAll() {
    this.renderCount += 1;
  }

  dispose() {
    this.disposed = true;
    if (typeof this.element.remove === 'function') {
      this.element.remove();
    }
  }
}

export const createdCanvases: MockCanvas[] = [];
(globalThis as any).__fabricCreatedCanvases = createdCanvases;

export const fabric = {
  Rect: MockRect,
  Canvas: MockCanvas,
  FabricObject: MockFabricObject,
};

export { MockRect as Rect, MockCanvas as Canvas, MockFabricObject as FabricObject };
