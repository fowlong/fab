// @ts-nocheck
export type TestChild = TestElement;

export class TestElement {
  public tagName: string;
  public children: TestChild[] = [];
  public parent: TestElement | null = null;
  public className = '';
  public style: Record<string, string> = {};
  public width = 0;
  public height = 0;
  private _innerHTML = '';

  constructor(tagName: string) {
    this.tagName = tagName.toUpperCase();
  }

  appendChild(child: TestChild) {
    child.parent = this;
    this.children.push(child);
    return child;
  }

  removeChild(child: TestChild) {
    this.children = this.children.filter((c) => c !== child);
    child.parent = null;
    return child;
  }

  remove() {
    if (this.parent) {
      this.parent.removeChild(this);
    }
  }

  get innerHTML() {
    return this._innerHTML;
  }

  set innerHTML(value: string) {
    this._innerHTML = value;
    if (value === '') {
      this.children.forEach((child) => {
        child.parent = null;
      });
      this.children = [];
    }
  }

  get childElementCount() {
    return this.children.length;
  }

  getContext(_type: string) {
    return {};
  }

  queryByTag(tag: string) {
    const upper = tag.toUpperCase();
    return this.children.filter((child) => child.tagName === upper);
  }
}

export class DocumentStub {
  public body = new TestElement('body');

  createElement(tag: string) {
    return new TestElement(tag);
  }
}

export function installDom() {
  const originalDocument = (globalThis as any).document;
  const document = new DocumentStub();
  (globalThis as any).document = document;
  return () => {
    if (originalDocument === undefined) {
      delete (globalThis as any).document;
    } else {
      (globalThis as any).document = originalDocument;
    }
  };
}
