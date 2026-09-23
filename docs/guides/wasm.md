---
title: WebAssembly Runtime
description: Use PolyXML's Rust XML/JSON core from Node.js, Bun, and browsers through WebAssembly.
---

# WebAssembly Runtime

The `polyxml-wasm` workspace crate builds the `@polyxml/wasm` npm package from
the same Rust core used by the native bindings. It works without a native Node
addon. Build the package from source with `./crates/polyxml-wasm/build.sh`.

```js
import { createPolyXml } from '@polyxml/wasm'

const polyxml = await createPolyXml()
const value = polyxml.xmlToJson('<order><qty>2</qty></order>')
const xml = polyxml.jsonToXml(value)
```

`xmlToJson` returns a JavaScript object; `jsonToXml` returns a string. For a
byte-oriented path, use `xmlToJsonBytes` and `jsonToXmlBytes`, which return
`Uint8Array`. Inputs may be strings, `Uint8Array`, or `ArrayBuffer`.

To apply XSD field types, compile a self-contained schema once:

```js
const schema = polyxml.schemaFromXsd(xsdText, 'order')
try {
  const order = schema.xmlToJson(xmlBytes)
  const output = schema.jsonToXmlBytes(order)
} finally {
  schema.free()
}
```

The inline XSD path rejects `include`, `import`, `redefine`, and `override`
because those need an external schema resolver. Browser bundlers load the
adjacent `.wasm` asset through the package's browser entry point; Node.js and
Bun use the Node entry point. `createPolyXml(source)` also accepts an explicit
Wasm source when the host controls asset loading.

`wasm-bindgen` copies byte inputs into Wasm memory and byte outputs back to
JavaScript memory. Converting JSON to a JavaScript object allocates again. The
current API handles complete documents; it does not incrementally process a
`ReadableStream`. End-to-end throughput, GC effects, and bundle tradeoffs
remain to be measured under [issue #47](https://github.com/nth-bailey/PolyXML/issues/47).
