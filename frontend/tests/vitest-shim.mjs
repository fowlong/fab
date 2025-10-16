import assert from 'node:assert/strict';
import { test } from 'node:test';

const suiteNames = [];
const afterEachStack = [];
const rootAfterEach = [];
const registeredMocks = new Set();

function describe(name, fn) {
  suiteNames.push(name);
  afterEachStack.push([]);
  try {
    fn();
  } finally {
    afterEachStack.pop();
    suiteNames.pop();
  }
}

function afterEach(hook) {
  if (afterEachStack.length === 0) {
    rootAfterEach.push(hook);
    return;
  }
  afterEachStack[afterEachStack.length - 1].push(hook);
}

async function runAfterEach() {
  for (let i = afterEachStack.length - 1; i >= 0; i -= 1) {
    for (const hook of afterEachStack[i]) {
      await hook();
    }
  }
  for (const hook of rootAfterEach) {
    await hook();
  }
}

function it(name, fn) {
  const parts = [...suiteNames, name].filter(Boolean);
  const fullName = parts.join(' › ');
  test(fullName, async () => {
    try {
      await fn();
    } finally {
      await runAfterEach();
    }
  });
}

function toStrictEqual(actual, expected) {
  assert.deepStrictEqual(actual, expected);
}

function expect(actual) {
  const matchers = {
    toEqual(expected) {
      toStrictEqual(actual, expected);
    },
    toBe(expected) {
      assert.strictEqual(actual, expected);
    },
    toBeCloseTo(expected, precision = 2) {
      const delta = Math.pow(10, -precision);
      assert.ok(Math.abs(actual - expected) <= delta, `${actual} not within ${delta} of ${expected}`);
    },
    toContain(expected) {
      if (typeof actual === 'string') {
        assert.ok(actual.includes(expected), `${actual} does not contain ${expected}`);
      } else if (Array.isArray(actual)) {
        assert.ok(actual.includes(expected), `array does not contain ${expected}`);
      } else {
        throw new Error('toContain expects a string or array');
      }
    },
    toBeInstanceOf(expected) {
      assert.ok(actual instanceof expected, `expected instance of ${expected.name}`);
    },
    toHaveBeenCalledTimes(expected) {
      assert.ok(actual?.mock?.calls, 'expected mock function');
      assert.strictEqual(actual.mock.calls.length, expected);
    },
    toHaveBeenCalledWith(...expectedArgs) {
      assert.ok(actual?.mock?.calls, 'expected mock function');
      const match = actual.mock.calls.some((args) => {
        try {
          toStrictEqual(args, expectedArgs);
          return true;
        } catch {
          return false;
        }
      });
      assert.ok(match, 'expected mock to be called with provided arguments');
    },
    toBeGreaterThan(expected) {
      assert.ok(actual > expected, `${actual} is not greater than ${expected}`);
    },
    get rejects() {
      if (!(actual instanceof Promise)) {
        throw new Error('rejects can only be used with Promises');
      }
      return {
        async toThrow(expected) {
          let thrown = false;
          try {
            await actual;
          } catch (err) {
            thrown = true;
            const message = err instanceof Error ? err.message : String(err);
            if (expected instanceof RegExp) {
              assert.ok(expected.test(message), `expected error to match ${expected}`);
            } else {
              assert.strictEqual(message, expected);
            }
          }
          if (!thrown) {
            throw new Error('expected promise to reject');
          }
        },
      };
    },
  };
  return matchers;
}

function createMock(implementation) {
  const mockFn = (...args) => {
    mockFn.mock.calls.push(args);
    if (implementation) {
      return implementation(...args);
    }
    return undefined;
  };
  mockFn.mock = { calls: [] };
  mockFn.mockResolvedValue = (value) => {
    implementation = () => Promise.resolve(value);
    return mockFn;
  };
  mockFn.mockReturnValue = (value) => {
    implementation = () => value;
    return mockFn;
  };
  mockFn.mockImplementation = (fn) => {
    implementation = fn;
    return mockFn;
  };
  registeredMocks.add(mockFn);
  return mockFn;
}

const vi = {
  fn(implementation) {
    return createMock(implementation);
  },
  restoreAllMocks() {
    for (const mockFn of registeredMocks) {
      mockFn.mock.calls = [];
    }
  },
};

export { afterEach, describe, expect, it, vi };
