# TypeScript/Wasm runtime benchmarks

Build the Wasm package, then install benchmark dependencies and run each tier:

```sh
./crates/polyxml-wasm/build.sh
cd benchmarks/typescript-wasm
npm ci
npm exec -- playwright install chromium-headless-shell
node --expose-gc bench.mjs micro
node --expose-gc bench.mjs medium
node --expose-gc bench.mjs large
bun bench.mjs micro
node browser-runner.mjs micro
```

Set `BENCH_CASE='polyxml object'` to run one Node/Bun case in isolation,
especially for the 50 MiB fixture. Set `BENCH_ITERATIONS=3` to shorten a
large-case exploratory run.

The fixture contains repeated simple records. The object cases return parsed
JavaScript values; the JSON bytes case measures transcoding before `JSON.parse`.
`xml2js` has different output semantics, so treat it as a throughput reference,
not a drop-in equivalent. Heap deltas are noisy, and GC events are reported for
the whole run. Record runtime, hardware, versions, and raw JSON alongside any
published results. The 50 MB tier requires substantial memory.
The browser lane uses headless Chromium, Vite, and Playwright. Browser `DOMParser`
returns a DOM rather than a plain object, so its timings are a separate parse
reference. Run each lane in a fresh process and repeat on an idle machine before
publishing performance claims. On a host Playwright does not support, install
a compatible Chromium binary and set `PLAYWRIGHT_CHROMIUM_EXECUTABLE` to its
path for `browser-runner.mjs`.
