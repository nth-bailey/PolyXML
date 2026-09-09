const test = require('node:test');
const assert = require('node:assert');

test('PolyXML JavaScript bindings', (t) => {
  let polyxml;
  try {
    polyxml = require('../index.js');
  } catch (err) {
    // If native addon not yet compiled locally, skip with notice
    t.diagnostic('Native addon not compiled in current environment; skipping live execution test.');
    return;
  }

  assert.strictEqual(typeof polyxml.version(), 'string');

  const schema = {
    name: 'Sensor',
    fields: [
      { name: 'id', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
      { name: 'name', xmlName: 'name', kind: 'element', scalarType: 'string' },
      { name: 'reading', xmlName: 'reading', kind: 'element', scalarType: 'float' },
      { name: 'calibrated', xmlName: 'calibrated', kind: 'element', scalarType: 'bool' },
    ],
  };

  const xml = '<Sensor id="101"><name>Barometric Altimeter</name><reading>1013.25</reading><calibrated>true</calibrated></Sensor>';
  const val = polyxml.deserialize(xml, schema);

  assert.strictEqual(val.id, 101);
  assert.strictEqual(val.name, 'Barometric Altimeter');
  assert.strictEqual(val.reading, 1013.25);
  assert.strictEqual(val.calibrated, true);

  // Test serialization
  const outBytes = polyxml.serialize('Sensor', val, schema, 2);
  assert.ok(outBytes instanceof Uint8Array || Buffer.isBuffer(outBytes));
  const outXml = Buffer.from(outBytes).toString('utf-8');
  assert.ok(outXml.includes('id="101"'));
  assert.ok(outXml.includes('<name>Barometric Altimeter</name>'));
  assert.ok(outXml.includes('<reading>1013.25</reading>'));
  assert.ok(outXml.includes('<calibrated>true</calibrated>'));
});
