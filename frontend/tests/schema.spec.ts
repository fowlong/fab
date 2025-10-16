import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

type Json = unknown;

type Schema = {
  $ref?: string;
  oneOf?: Schema[];
  type?: string;
  const?: Json;
  enum?: Json[];
  properties?: Record<string, Schema>;
  required?: string[];
  items?: Schema;
  minItems?: number;
  maxItems?: number;
  minimum?: number;
  maximum?: number;
  additionalProperties?: boolean;
};

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const schemaDir = resolve(__dirname, '../../shared/schema');

function parseJson(relativePath: string): Json {
  const fullPath = join(__dirname, relativePath);
  return JSON.parse(readFileSync(fullPath, 'utf8'));
}

function resolveRef(ref: string, root: any): Schema {
  if (!ref.startsWith('#/')) {
    throw new Error(`Unsupported ref: ${ref}`);
  }
  const parts = ref.slice(2).split('/');
  let current: any = root;
  for (const part of parts) {
    current = current?.[part];
  }
  if (!current) {
    throw new Error(`Unable to resolve ref: ${ref}`);
  }
  return current as Schema;
}

function isObject(value: Json): value is Record<string, Json> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function validateSchema(schema: Schema, value: Json, root: any, path: string): string[] {
  if (schema.$ref) {
    return validateSchema(resolveRef(schema.$ref, root), value, root, path);
  }

  if (schema.oneOf) {
    const results = schema.oneOf.map((candidate) => validateSchema(candidate, value, root, path));
    if (results.some((errors) => errors.length === 0)) {
      return [];
    }
    return results.flat();
  }

  const errors: string[] = [];

  if (schema.type) {
    switch (schema.type) {
      case 'object':
        if (!isObject(value)) {
          return [`${path} expected object`];
        }
        break;
      case 'array':
        if (!Array.isArray(value)) {
          return [`${path} expected array`];
        }
        break;
      case 'number':
        if (typeof value !== 'number') {
          return [`${path} expected number`];
        }
        break;
      case 'integer':
        if (typeof value !== 'number' || !Number.isInteger(value)) {
          return [`${path} expected integer`];
        }
        break;
      case 'string':
        if (typeof value !== 'string') {
          return [`${path} expected string`];
        }
        break;
      case 'boolean':
        if (typeof value !== 'boolean') {
          return [`${path} expected boolean`];
        }
        break;
    }
  }

  if (schema.const !== undefined && value !== schema.const) {
    errors.push(`${path} expected constant ${schema.const}`);
  }

  if (schema.enum && !schema.enum.some((entry) => entry === value)) {
    errors.push(`${path} expected one of ${schema.enum.join(', ')}`);
  }

  if (Array.isArray(value)) {
    if (schema.minItems !== undefined && value.length < schema.minItems) {
      errors.push(`${path} expected at least ${schema.minItems} items`);
    }
    if (schema.maxItems !== undefined && value.length > schema.maxItems) {
      errors.push(`${path} expected at most ${schema.maxItems} items`);
    }
    if (schema.items) {
      value.forEach((item, index) => {
        errors.push(...validateSchema(schema.items as Schema, item, root, `${path}/${index}`));
      });
    }
  }

  if (schema.minimum !== undefined && typeof value === 'number' && value < schema.minimum) {
    errors.push(`${path} expected >= ${schema.minimum}`);
  }

  if (schema.maximum !== undefined && typeof value === 'number' && value > schema.maximum) {
    errors.push(`${path} expected <= ${schema.maximum}`);
  }

  if (schema.properties && isObject(value)) {
    const keys = Object.keys(value);
    if (schema.required) {
      for (const key of schema.required) {
        if (!(key in value)) {
          errors.push(`${path}/${key} is required`);
        }
      }
    }
    if (schema.additionalProperties === false) {
      const allowed = new Set(Object.keys(schema.properties));
      for (const key of keys) {
        if (!allowed.has(key)) {
          errors.push(`${path}/${key} is not allowed`);
        }
      }
    }
    for (const key of keys) {
      const propertySchema = schema.properties[key];
      if (propertySchema) {
        errors.push(...validateSchema(propertySchema, value[key], root, `${path}/${key}`));
      }
    }
  }

  return errors;
}

function validateAgainstSchema(rootSchema: any, value: Json): string[] {
  return validateSchema(rootSchema as Schema, value, rootSchema, '#');
}

describe('shared JSON schemas', () => {
  it('validates the sample IR payload', () => {
    const schema = JSON.parse(readFileSync(join(schemaDir, 'ir.schema.json'), 'utf8'));
    const sample = parseJson('fixtures/ir.sample.json');
    const errors = validateAgainstSchema(schema, sample);

    expect(errors).toEqual([]);
  });

  it('validates the sample patch payload', () => {
    const schema = JSON.parse(readFileSync(join(schemaDir, 'patch.schema.json'), 'utf8'));
    const sample = parseJson('fixtures/patch.sample.json');
    const errors = validateAgainstSchema(schema, sample);

    expect(errors).toEqual([]);
  });

  it('rejects invalid patch payloads', () => {
    const schema = JSON.parse(readFileSync(join(schemaDir, 'patch.schema.json'), 'utf8'));
    const invalid = [{ op: 'transform', target: { page: 0 } }];
    const errors = validateAgainstSchema(schema, invalid);

    expect(errors.length).toBeGreaterThan(0);
  });
});
