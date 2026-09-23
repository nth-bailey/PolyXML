# `@polyxml/wasm`

WebAssembly XML/JSON conversion for Node.js, Bun, and browsers. This package
uses PolyXML's Rust core without a native Node addon.

```js
import { createPolyXml } from '@polyxml/wasm'

const polyxml = await createPolyXml()
const value = polyxml.xmlToJson('<order><qty>2</qty></order>')
const xml = polyxml.jsonToXml(value)
```

The browser entry point loads the adjacent `.wasm` asset with `fetch`; Node.js
and Bun load it from the package. An application can pass Wasm bytes or a URL
to `createPolyXml(source)` to control loading. `xmlToJsonBytes` and
`jsonToXmlBytes` return `Uint8Array` instead of converting to JS values/text.

For typed conversion, compile a self-contained XSD once:

```js
const schema = polyxml.schemaFromXsd(xsdText, 'order')
const order = schema.xmlToJson(xmlBytes)
const xmlBytes = schema.jsonToXmlBytes(order)
schema.free()
```

Inline XSDs with `include`, `import`, `redefine`, or `override` are rejected;
browser runtimes cannot read referenced schema files. Schema-free conversion
is dynamic and does not validate against an XSD.

Input `Uint8Array` values are copied into Wasm memory by `wasm-bindgen`, and
output bytes are copied back into JS memory. Converting to JS objects incurs
further allocations. The API does not promise zero-copy behavior or a speedup
over native JS parsers.

## Build from source

Install the `wasm32-unknown-unknown` Rust target and `wasm-pack`, then run:

```bash
./crates/polyxml-wasm/build.sh
node --test crates/polyxml-wasm/test/*.test.mjs
```

The publishable package is generated in `crates/polyxml-wasm/pkg/`.
