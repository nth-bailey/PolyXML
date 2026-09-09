---
title: TypeScript & Node.js Guide
description: Fast XML parsing in TypeScript and Node.js using napi-rs.
---

# TypeScript & Node.js Guide

PolyXML provides native C-speed XML processing for Node.js via `napi-rs`.

## Installation

```bash
npm install polyxml
```

## Example Usage

```typescript
import { deserialize, serialize, ModelSchema } from 'polyxml';

const schema: ModelSchema = {
  name: 'Device',
  fields: [
    { name: 'id', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
    { name: 'status', xmlName: 'status', kind: 'element', scalarType: 'string' },
  ],
};

const xml = '<Device id="42"><status>ONLINE</status></Device>';

// 1. Deserialize directly into JavaScript object
const device = deserialize(xml, schema);
console.log(`Device ID: ${device.id}, Status: ${device.status}`);

// 2. Serialize back to XML
const xmlBytes = serialize('Device', device, schema, 2);
```
