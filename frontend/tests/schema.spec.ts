import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { sampleIr } from './__fixtures__/sampleIr';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const schemaDir = path.resolve(__dirname, '..', '..', 'shared', 'schema');

function loadSchema(name: string) {
  const schemaPath = path.join(schemaDir, name);
  return JSON.parse(readFileSync(schemaPath, 'utf8'));
}

function resolveRef(root: any, reference: string) {
  if (!reference.startsWith('#/')) {
    throw new Error(`Unsupported $ref ${reference}`);
  }
  return reference
    .slice(2)
    .split('/')
    .reduce((current, part) => {
      if (current?.[part] === undefined) {
        throw new Error(`Unresolved $ref ${reference}`);
      }
      return current[part];
    }, root);
}

function matchesType(type: any, value: unknown): boolean {
  if (Array.isArray(type)) {
    return type.some((entry) => matchesType(entry, value));
  }
  switch (type) {
    case 'object':
      return typeof value === 'object' && value !== null && !Array.isArray(value);
    case 'array':
      return Array.isArray(value);
    case 'number':
      return typeof value === 'number';
    case 'integer':
      return typeof value === 'number' && Number.isInteger(value);
    case 'string':
      return typeof value === 'string';
    case 'boolean':
      return typeof value === 'boolean';
    default:
      return true;
  }
}

function validate(schema: any, data: any, root: any = schema): boolean {
  if (schema.$ref) {
    return validate(resolveRef(root, schema.$ref), data, root);
  }

  if (schema.oneOf) {
    return schema.oneOf.some((variant: any) => validate(variant, data, root));
  }

  if (schema.const !== undefined) {
    return JSON.stringify(schema.const) === JSON.stringify(data);
  }

  if (schema.enum) {
    if (!schema.enum.some((value: any) => JSON.stringify(value) === JSON.stringify(data))) {
      return false;
    }
  }

  if (schema.type && !matchesType(schema.type, data)) {
    return false;
  }

  if (typeof data === 'number') {
    if (schema.minimum !== undefined && data < schema.minimum) {
      return false;
    }
    if (schema.maximum !== undefined && data > schema.maximum) {
      return false;
    }
  }

  if (Array.isArray(data)) {
    if (schema.minItems !== undefined && data.length < schema.minItems) {
      return false;
    }
    if (schema.maxItems !== undefined && data.length > schema.maxItems) {
      return false;
    }
    if (schema.items) {
      return data.every((item) => validate(schema.items, item, root));
    }
  }

  if (data && typeof data === 'object' && !Array.isArray(data)) {
    if (schema.required) {
      for (const key of schema.required) {
        if (!(key in data)) {
          return false;
        }
      }
    }

    if (schema.properties) {
      for (const [key, value] of Object.entries(data)) {
        if (schema.properties[key]) {
          if (!validate(schema.properties[key], value, root)) {
            return false;
          }
        } else if (schema.additionalProperties === false) {
          return false;
        }
      }
    }
  }

  return true;
}

describe('shared JSON schemas', () => {
  it('validates the bundled sample IR', () => {
    const schema = loadSchema('ir.schema.json');
    expect(validate(schema, sampleIr, schema)).toBe(true);
  });

  it('validates patch operations and rejects invalid payloads', () => {
    const schema = loadSchema('patch.schema.json');
    const ops = [
      {
        op: 'transform',
        target: { page: 0, id: 't:42' },
        kind: 'text',
        deltaMatrixPt: [1, 0, 0, 1, 4, -3],
      },
    ];
    expect(validate(schema, ops, schema)).toBe(true);

    const invalid = [{ op: 'transform', target: { page: -1, id: 7 } }];
    expect(validate(schema, invalid, schema)).toBe(false);
  });
});
