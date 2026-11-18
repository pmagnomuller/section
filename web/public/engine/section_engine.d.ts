/* tslint:disable */
/* eslint-disable */

export class Handle {
    free(): void;
    [Symbol.dispose](): void;
    catalog(): string;
    cmd(json: string): void;
    export_doc(): string;
    hud(): string;
    constructor(sample_rate: number);
    /**
     * Interleaved stereo frames, left/right pairs, in wasm memory.
     */
    render(frames: number): number;
}

export function take_memory(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_handle_free: (a: number, b: number) => void;
    readonly handle_catalog: (a: number) => [number, number];
    readonly handle_cmd: (a: number, b: number, c: number) => [number, number];
    readonly handle_export_doc: (a: number) => [number, number];
    readonly handle_hud: (a: number) => [number, number];
    readonly handle_new: (a: number) => number;
    readonly handle_render: (a: number, b: number) => number;
    readonly take_memory: () => any;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
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
