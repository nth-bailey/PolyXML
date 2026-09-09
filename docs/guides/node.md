---
title: TypeScript & Node.js Guide
description: High-throughput XML serialization and deserialization in TypeScript and Node.js using native Rust Node-API (napi-rs).
---

# TypeScript & Node.js Guide

PolyXML provides native compiled C/Rust performance directly inside the Node.js event loop via `napi-rs`. It avoids the CPU bottlenecks of JavaScript-based XML parsers (e.g. `xml2js`, `fast-xml-parser`) by executing schema validation and XML parsing in compiled Rust.

---

## 📦 Installation

```bash
npm install polyxml
```

PolyXML ships pre-built binaries across Linux, macOS, and Windows via standard npm optional dependencies.

---

## 1. Strongly-Typed TypeScript Schemas

Define your domain models and corresponding `ModelSchema` definitions:

```typescript
import { deserialize, serialize, ModelSchema } from 'polyxml';

// 1. Define TypeScript interface
interface DeviceTelemetry {
  deviceId: number;
  hostname: string;
  uptimeSeconds: number;
  isOnline: boolean;
  cpuLoad: number;
}

// 2. Define corresponding PolyXML ModelSchema
const telemetrySchema: ModelSchema = {
  name: 'DeviceTelemetry',
  fields: [
    { name: 'deviceId', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
    { name: 'hostname', xmlName: 'host', kind: 'element', scalarType: 'string' },
    { name: 'uptimeSeconds', xmlName: 'uptime', kind: 'element', scalarType: 'int' },
    { name: 'isOnline', xmlName: 'online', kind: 'element', scalarType: 'bool' },
    { name: 'cpuLoad', xmlName: 'cpu', kind: 'element', scalarType: 'float' },
  ],
};

const xml = `
<DeviceTelemetry id="88">
  <host>edge-gw-01</host>
  <uptime>864000</uptime>
  <online>true</online>
  <cpu>0.42</cpu>
</DeviceTelemetry>
`;

// 3. Deserialize directly into typed JavaScript object
const telem = deserialize(xml, telemetrySchema) as unknown as DeviceTelemetry;
console.log(`Device ${telem.deviceId} (${telem.hostname}): Online=${telem.isOnline}, CPU=${telem.cpuLoad}`);

// 4. Serialize back to formatted XML bytes (Uint8Array)
const xmlBytes = serialize('DeviceTelemetry', telem, telemetrySchema, 2);
console.log(Buffer.from(xmlBytes).toString('utf-8'));
```

---

## 2. Parsing Binary Buffers (`Uint8Array` / `Buffer`)

PolyXML accepts both JavaScript `string` and raw `Uint8Array` / `Buffer` inputs without converting to UTF-16 strings first:

```typescript
import * as fs from 'node:fs';
import { deserialize, ModelSchema } from 'polyxml';

const catalogSchema: ModelSchema = {
  name: 'Product',
  fields: [
    { name: 'sku', xmlName: 'sku', kind: 'attribute', scalarType: 'string' },
    { name: 'title', xmlName: 'title', kind: 'element', scalarType: 'string' },
    { name: 'price', xmlName: 'price', kind: 'element', scalarType: 'float' },
  ],
};

// Read XML file directly as a Buffer
const buffer: Buffer = fs.readFileSync('product.xml');

// Pass Buffer directly into Rust engine with zero UTF-16 string conversion
const product = deserialize(buffer, catalogSchema);
console.log('Parsed product from buffer:', product);
```

---

## 3. High-Throughput HTTP Service (Express / Fastify)

Integrate PolyXML into Express or Fastify request handlers to process high volumes of inbound XML webhooks:

```typescript
import express, { Request, Response } from 'express';
import { deserialize, serialize, ModelSchema } from 'polyxml';

const orderSchema: ModelSchema = {
  name: 'PurchaseOrder',
  fields: [
    { name: 'orderId', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
    { name: 'customer', xmlName: 'customer', kind: 'element', scalarType: 'string' },
    { name: 'amount', xmlName: 'amount', kind: 'element', scalarType: 'float' },
  ],
};

const app = express();

// Use express.raw to receive unparsed XML Buffers
app.post('/api/orders', express.raw({ type: 'application/xml' }), (req: Request, res: Response) => {
  try {
    const order = deserialize(req.body, orderSchema) as any;
    console.log(`Processed Order #${order.orderId} for ${order.customer} ($${order.amount})`);

    // Return XML acknowledgment
    const ack = { status: 'ACCEPTED', orderId: order.orderId };
    const ackSchema: ModelSchema = {
      name: 'OrderAck',
      fields: [
        { name: 'status', xmlName: 'status', kind: 'element', scalarType: 'string' },
        { name: 'orderId', xmlName: 'orderId', kind: 'element', scalarType: 'int' },
      ],
    };

    const ackXml = serialize('OrderAck', ack, ackSchema, 0);
    res.setHeader('Content-Type', 'application/xml');
    res.send(Buffer.from(ackXml));
  } catch (err: any) {
    res.status(400).json({ error: 'Malformed XML payload', details: err.message });
  }
});

app.listen(3000, () => console.log('XML Server listening on http://localhost:3000'));
```

---

## 4. Error Handling & Validation

When parsing malformed XML or invalid scalar values (such as non-numeric characters in an integer element), PolyXML throws informative native errors:

```typescript
import { deserialize, ModelSchema } from 'polyxml';

const schema: ModelSchema = {
  name: 'Data',
  fields: [
    { name: 'count', xmlName: 'count', kind: 'element', scalarType: 'int' },
  ],
};

try {
  // Invalid integer scalar "not-a-number"
  deserialize('<Data><count>not-a-number</count></Data>', schema);
} catch (err: any) {
  console.error('Caught validation error:', err.message);
  // Output: Caught validation error: Failed to parse scalar for field count
}
```

---

## 5. Performance Best Practices for Node.js

1. **Avoid Converting Buffers to Strings**: If your XML arrives via HTTP or disk as a `Buffer`, pass it directly to `deserialize(buffer, schema)`. Converting `buffer.toString('utf-8')` forces V8 to allocate a UTF-16 string on the V8 heap unnecessarily.
2. **Reuse `ModelSchema` Objects**: Schema definitions should be instantiated once as top-level constants rather than recreated inside request handlers.
3. **Prefer Compact Serialization for APIs**: When sending XML over network APIs, omit the `indent` argument (`serialize(root, val, schema)`) to minimize bandwidth and skip whitespace generation.
