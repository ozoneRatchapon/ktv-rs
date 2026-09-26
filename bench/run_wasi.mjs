// Run a wasm32-wasip1 benchmark binary under Node's WASI (V8, same engine as Chrome).
// Usage: node bench/run_wasi.mjs <path/to/bench.wasm>
import { readFile } from 'node:fs/promises';
import { WASI } from 'node:wasi';

const wasi = new WASI({ version: 'preview1', args: ['bench'], env: {} });
const module = await WebAssembly.compile(await readFile(process.argv[2]));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
wasi.start(instance);
