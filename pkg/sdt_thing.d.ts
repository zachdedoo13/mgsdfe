/* tslint:disable */
/* eslint-disable */
/**
* Your handle to the web app from JavaScript.
*/
export class WebHandle {
  free(): void;
/**
* Installs a panic hook, then returns.
*/
  constructor();
/**
* Call this once from JavaScript to start your app.
* @param {string} canvas_id
* @returns {Promise<void>}
*/
  start(canvas_id: string): Promise<void>;
/**
* Shut down eframe and clean up resources.
*/
  destroy(): void;
/**
* The JavaScript can check whether or not your app has crashed:
* @returns {boolean}
*/
  has_panicked(): boolean;
/**
* @returns {string | undefined}
*/
  panic_message(): string | undefined;
/**
* @returns {string | undefined}
*/
  panic_callstack(): string | undefined;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_webhandle_free: (a: number, b: number) => void;
  readonly webhandle_new: () => number;
  readonly webhandle_start: (a: number, b: number, c: number) => number;
  readonly webhandle_destroy: (a: number) => void;
  readonly webhandle_has_panicked: (a: number) => number;
  readonly webhandle_panic_message: (a: number, b: number) => void;
  readonly webhandle_panic_callstack: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4d0dca79584001e0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h35f9d1cc3b6e6f92: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7643d960fd7beb05: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2ec1cfc9dbe1d356: (a: number, b: number, c: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h0b684dfffed5d693: (a: number, b: number, c: number, d: number) => void;
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
