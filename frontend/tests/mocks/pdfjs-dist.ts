// @ts-nocheck
export const pdf = {
  numPages: 1,
  async getPage() {
    return {
      getViewport: () => ({ width: 200, height: 300 }),
      render: () => ({ promise: Promise.resolve() }),
    };
  },
};

export const GlobalWorkerOptions = { workerSrc: '' };

export function getDocument(_options: unknown) {
  return { promise: Promise.resolve(pdf) };
}
