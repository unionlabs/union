/* tslint:disable */
/* eslint-disable */
export function decode_packet(packet: Uint8Array): any;
export function encode_packet(packet: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly decode_packet: (a: number, b: number, c: number) => void;
  readonly encode_packet: (a: number, b: number) => void;
  readonly allocate: (a: number) => number;
  readonly deallocate: (a: number) => void;
  readonly interface_version_8: () => void;
  readonly requires_cosmwasm_1_1: () => void;
  readonly requires_cosmwasm_1_2: () => void;
  readonly requires_cosmwasm_1_3: () => void;
  readonly requires_cosmwasm_1_4: () => void;
  readonly requires_cosmwasm_2_0: () => void;
  readonly requires_iterator: () => void;
  readonly requires_staking: () => void;
  readonly requires_stargate: () => void;
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
