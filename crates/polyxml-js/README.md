# PolyXML Node.js & TypeScript Bindings

High-performance, polyglot streaming XML data-binding engine for Node.js and TypeScript.

- Precompiled native binaries for Linux (x86_64), macOS (Apple Silicon & Intel), and Windows (x64).
- Zero external native toolchain dependencies needed at runtime.
- Powered by the ultra-fast Rust `polyxml-core` engine and NAPI-RS.

## Installation

```bash
npm install polyxml
```

## Usage

```javascript
const polyxml = require('polyxml');

const schema = {
  name: 'Book',
  fields: [
    { name: 'id', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
    { name: 'title', xmlName: 'title', kind: 'element', scalarType: 'string' },
    { name: 'price', xmlName: 'price', kind: 'element', scalarType: 'float' }
  ]
};

const xml = '<Book id="42"><title>Rust in Action</title><price>39.99</price></Book>';
const book = polyxml.deserialize(xml, schema);

console.log(book);
// { id: 42, title: 'Rust in Action', price: 39.99 }
```

## License

MIT
