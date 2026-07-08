// AttributeValue mirrors the backend's untagged serde encoding: scalars,
// arrays, and nested maps arrive as plain JSON values.
export type AttributeValue =
  | string
  | number
  | boolean
  | AttributeValue[]
  | { [key: string]: AttributeValue };

export type Attributes = Record<string, AttributeValue>;

export interface InstrumentationScope {
  name: string;
  version?: string;
  attributes?: Attributes;
}
