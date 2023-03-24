import * as wasm from "./nass_bg.wasm";
import { __wbg_set_wasm } from "./nass_bg.js";
__wbg_set_wasm(wasm);
export * from "./nass_bg.js";
