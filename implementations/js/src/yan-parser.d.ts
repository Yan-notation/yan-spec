/**
 * Type definitions for yan-notation
 * YAN (Yet Another Notation) parser for JavaScript/TypeScript
 */

/**
 * A value produced by a `@<type>` type hint that doesn't map to a native
 * JS primitive (e.g. `@date`, `@url`, `@uuid`). Plain values like
 * `@int`/`@float`/`@bool` are returned as native number/boolean instead.
 */
export interface YANTypedValue {
  __type: string;
  __value: string;
}

/** Any scalar value YAN can produce. */
export type YANScalar = string | number | boolean | null | YANTypedValue;

/** Any value YAN can produce, including nested arrays/objects. */
export type YANValue =
  | YANScalar
  | YANValue[]
  | { [key: string]: YANValue };

/** Root document shape returned by `YANParser.parse()`. */
export type YANDocument = { [key: string]: YANValue } | YANValue[] | YANScalar;

export interface YANParserOptions {
  /** Reject ambiguous/non-canonical syntax instead of best-effort parsing. Default: false */
  strict?: boolean;
  /** Preserve comments when round-tripping (not used by `parse`/`stringify` directly). Default: false */
  preserveComments?: boolean;
}

export interface YANStringifyOptions {
  /** Number of spaces per indentation level. Default: 2 */
  indent?: number;
  /** Starting indentation level. Default: 0 */
  level?: number;
  /** Internal: render as an inline `{ ... }` block instead of indented lines. */
  inline?: boolean;
}

export class YANParser {
  constructor(options?: YANParserOptions);

  /** Parse a YAN source string into a JavaScript value. */
  parse(source: string): YANDocument;

  /** Convert a JavaScript value back into a YAN string. */
  stringify(value: YANValue, options?: YANStringifyOptions): string;
}

export class YANParseError extends Error {
  constructor(message: string);
  name: 'YANParseError';
}
