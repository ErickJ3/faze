export type AttributeValue =
  | { type: "string"; value: string }
  | { type: "int"; value: number }
  | { type: "double"; value: number }
  | { type: "bool"; value: boolean }
  | { type: "bytes"; value: number[] }
  | { type: "array"; value: AttributeValue[] };

export type Attributes = Record<string, AttributeValue>;
