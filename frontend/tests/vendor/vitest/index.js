import assert from 'node:assert/strict';
import {
  afterEach as nodeAfterEach,
  beforeEach as nodeBeforeEach,
  describe as nodeDescribe,
  it as nodeIt,
  test as nodeTest,
} from 'node:test';

export const describe = nodeDescribe;
export const it = nodeIt;
export const test = nodeTest;
export const beforeEach = nodeBeforeEach;
export const afterEach = nodeAfterEach;

class Expectation {
  constructor(actual) {
    this.actual = actual;
  }

  toEqual(expected) {
    assert.deepStrictEqual(this.actual, expected);
  }

  toBe(expected) {
    assert.strictEqual(this.actual, expected);
  }

  toHaveBeenCalledTimes(count) {
    assert.ok(this.actual?.mock?.calls, 'expected a mock function');
    assert.strictEqual(this.actual.mock.calls.length, count);
  }

  toBeCloseTo(expected, precision = 2) {
    const diff = Math.abs(this.actual - expected);
    const tolerance = Math.pow(10, -precision);
    assert.ok(diff <= tolerance, `${this.actual} is not within ${tolerance} of ${expected}`);
  }

  toHaveProperty(key) {
    assert.ok(this.actual != null && key in this.actual, `expected property ${key}`);
  }

  toBeInstanceOf(ctor) {
    assert.ok(this.actual instanceof ctor, `expected instance of ${ctor?.name ?? 'constructor'}`);
  }
}

export function expect(actual) {
  if (actual && typeof actual.then === 'function') {
    return {
      rejects: {
        async toThrow(matcher) {
          try {
            await actual;
            assert.fail('expected promise to reject');
          } catch (err) {
            if (matcher instanceof RegExp) {
              assert.match(String(err), matcher);
            } else if (typeof matcher === 'string') {
              assert.strictEqual(String(err), matcher);
            } else {
              assert.ok(err instanceof matcher, `expected rejection to be instance of ${matcher?.name ?? 'Error'}`);
            }
          }
        },
      },
    };
  }
  return new Expectation(actual);
}

const stubs = [];

export const vi = {
  fn(impl = () => {}) {
    const calls = [];
    const mockFn = (...args) => {
      calls.push(args);
      return mockFn._impl(...args);
    };
    mockFn._impl = impl;
    mockFn.mock = { calls };
    mockFn.mockImplementation = (fn) => {
      mockFn._impl = fn;
      return mockFn;
    };
    mockFn.mockReturnValue = (value) => {
      mockFn._impl = () => value;
      return mockFn;
    };
    mockFn.mockResolvedValue = (value) => {
      mockFn._impl = () => Promise.resolve(value);
      return mockFn;
    };
    return mockFn;
  },
  stubGlobal(name, value) {
    const exists = Object.prototype.hasOwnProperty.call(globalThis, name);
    stubs.push({ name, exists, value: exists ? globalThis[name] : undefined });
    globalThis[name] = value;
  },
  restoreAllMocks() {
    while (stubs.length) {
      const { name, exists, value } = stubs.pop();
      if (exists) {
        globalThis[name] = value;
      } else {
        delete globalThis[name];
      }
    }
  },
};
