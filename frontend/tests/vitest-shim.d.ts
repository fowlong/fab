declare module 'vitest' {
  export function describe(name: string, fn: () => void | Promise<void>): void;
  export function it(name: string, fn: () => void | Promise<void>): void;
  export function afterEach(fn: () => void | Promise<void>): void;
  export function expect<T>(value: T): Expectation<T> & PromiseMatchers;
  export const vi: {
    fn<T extends (...args: any[]) => any>(impl?: T): ViMock<T>;
    restoreAllMocks(): void;
  };

  interface ViMock<T extends (...args: any[]) => any> {
    (...args: Parameters<T>): ReturnType<T>;
    mock: {
      calls: Parameters<T>[];
    };
    mockResolvedValue(value: any): ViMock<T>;
    mockReturnValue(value: any): ViMock<T>;
    mockImplementation(impl: T): ViMock<T>;
  }

  interface Expectation<T> {
    toEqual(expected: any): void;
    toBe(expected: any): void;
    toBeCloseTo(expected: number, precision?: number): void;
    toContain(expected: any): void;
    toBeInstanceOf(expected: new (...args: any[]) => any): void;
    toHaveBeenCalledTimes(expected: number): void;
    toHaveBeenCalledWith(...expected: any[]): void;
    toBeGreaterThan(expected: number): void;
  }

  interface PromiseMatchers {
    rejects: {
      toThrow(expected: RegExp | string): Promise<void>;
    };
  }
}
