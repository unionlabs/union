/* tslint:disable */
/* eslint-disable */
/**
 * bytes -> packet
 */
export function decode_packet(packet: Uint8Array): any;
/**
 * packet -> bytes
 */
export function encode_packet(packet: any): any;
/**
 * bytes -> instruction
 */
export function decode_instruction(instruction: Uint8Array): any;
/**
 * instruction -> bytes
 */
export function encode_instruction(instruction: any): any;
/**
 * instruction -> shape
 */
export function packet_shape(instruction: any): any;
/**
 * (shape, bytes) -> ack
 */
export function decode_ack(shape: any, ack: Uint8Array): any;
/**
 * ack -> bytes
 */
export function encode_ack(ack: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly decode_ack: (a: number, b: number, c: number, d: number) => void;
  readonly decode_instruction: (a: number, b: number, c: number) => void;
  readonly decode_packet: (a: number, b: number, c: number) => void;
  readonly encode_ack: (a: number, b: number) => void;
  readonly encode_instruction: (a: number, b: number) => void;
  readonly encode_packet: (a: number, b: number) => void;
  readonly packet_shape: (a: number, b: number) => void;
  readonly commit_hash: (a: number) => void;
  readonly __wbindgen_export_0: (a: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
