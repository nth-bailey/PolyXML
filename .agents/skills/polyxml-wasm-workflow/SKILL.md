---
name: polyxml-wasm-workflow
description: Use when changing the polyxml-wasm package, its record streaming API, or the Node/Bun/browser XML benchmarks.
---

# PolyXML Wasm workflow

Build with `./crates/polyxml-wasm/build.sh`; it runs `wasm-pack` and copies the
maintained wrappers, declarations, and stream helper into `pkg/`. The generated
`pkg/` directory is ignored by Git. Run `node --test
crates/polyxml-wasm/test/wasm.test.mjs` after building. The CI Wasm job also
runs Bun tests. `index.node.js` loads a local Wasm file; `index.js` uses the
browser entry point and accepts an explicit source.

The complete-document methods use the Rust transcoder. `parseStream` emits
one direct child of an XML root at a time using the schema-free converter.
Chunk boundaries can split UTF-8 code points, tags, quoted attributes, CDATA,
and references. Keep tests for these boundaries and early emission. Do not
claim zero-copy: `wasm-bindgen` transfers bytes by copying.

The cross-runtime benchmark lives in `benchmarks/typescript-wasm`. Install
with `npm ci` after building the package, then run `bench.mjs` with Node or Bun
and `browser-runner.mjs` with headless Chromium. The 50 MiB fixture can consume
substantial memory; use `BENCH_CASE` and `BENCH_ITERATIONS` for isolated runs.
Compare object-returning parsers separately from byte output or DOM output.
Publish machine and runtime versions, fixture shape, case selection, and raw
limitations with any figures in `docs/benchmarks/wasm-vs-js.md`.
