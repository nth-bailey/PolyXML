/**
 * PolyXML TypeScript Type Definitions
 */

export type FieldKind = 'attribute' | 'element' | 'text';

export type ScalarType =
  | 'string'
  | 'int'
  | 'float'
  | 'bool'
  | 'decimal'
  | 'xml_date'
  | 'xml_datetime'
  | 'any';

export interface FieldDefinition {
  name: string;
  xmlName: string;
  kind: FieldKind;
  scalarType: ScalarType;
}

export interface ModelSchema {
  name: string;
  fields: FieldDefinition[];
}

export type PolyValue =
  | null
  | boolean
  | number
  | string
  | PolyValue[]
  | { [key: string]: PolyValue };

/**
 * Deserialize an XML string or Uint8Array into a JavaScript object based on schema.
 */
export function deserialize(
  xml: string | Uint8Array,
  schema: ModelSchema
): PolyValue;

/**
 * Serialize a JavaScript object into XML bytes based on schema.
 */
export function serialize(
  rootName: string,
  value: Record<string, any>,
  schema: ModelSchema,
  indent?: number | null
): Uint8Array;

/**
 * Return the PolyXML engine version.
 */
export function version(): string;
