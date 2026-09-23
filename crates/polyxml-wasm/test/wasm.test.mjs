import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readFile } from 'node:fs/promises'
import { createPolyXml } from '../pkg/index.node.js'
import { createPolyXml as createWebPolyXml } from '../pkg/index.js'

const polyxml = await createPolyXml()

test('schema-free XML and JSON conversion', () => {
  const value = polyxml.xmlToJson('<root><name>Ada &amp; Bob</name><active>true</active></root>')
  assert.deepEqual(value, { root: { active: true, name: 'Ada & Bob' } })
  const xml = polyxml.jsonToXml(value)
  assert.match(xml, /<name>Ada &amp; Bob<\/name>/)
  assert.deepEqual(polyxml.xmlToJson(new TextEncoder().encode(xml)), value)
  assert.ok(polyxml.xmlToJsonBytes(xml) instanceof Uint8Array)
})

test('XSD-backed conversion keeps scalar types and the chosen root name', () => {
  const xsd = '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">' +
    '<xs:complexType name="Item"><xs:sequence><xs:element name="qty" type="xs:int"/>' +
    '</xs:sequence></xs:complexType><xs:element name="item" type="Item"/></xs:schema>'
  const schema = polyxml.schemaFromXsd(xsd, 'item')
  try {
    assert.deepEqual(schema.xmlToJson('<item><qty>42</qty></item>'), { qty: 42 })
    assert.equal(schema.jsonToXml({ qty: 42 }), '<item><qty>42</qty></item>')
  } finally {
    schema.free()
  }
})

test('XSD-backed conversion handles nested repeated elements', () => {
  const xsd = '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">' +
    '<xs:complexType name="Item"><xs:sequence><xs:element name="qty" type="xs:int"/>' +
    '</xs:sequence></xs:complexType><xs:complexType name="Order"><xs:sequence>' +
    '<xs:element name="item" type="Item" maxOccurs="unbounded"/>' +
    '</xs:sequence></xs:complexType><xs:element name="order" type="Order"/></xs:schema>'
  const schema = polyxml.schemaFromXsd(xsd, 'order')
  try {
    const order = schema.xmlToJson('<order><item><qty>2</qty></item><item><qty>3</qty></item></order>')
    assert.deepEqual(order, { item: [{ qty: 2 }, { qty: 3 }] })
    assert.deepEqual(schema.xmlToJson(schema.jsonToXml(order)), order)
  } finally {
    schema.free()
  }
})

test('external schema references and invalid input fail clearly', () => {
  const imported = '<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">' +
    '<xs:include schemaLocation="other.xsd"/></xs:schema>'
  assert.throws(() => polyxml.schemaFromXsd(imported), /Inline XSD must not use/)
  assert.throws(() => polyxml.xmlToJson('<root>'), /Empty XML input/)
})

test('the browser entry accepts explicit Wasm bytes', async () => {
  const wasm = await readFile(new URL('../pkg/polyxml_wasm_bg.wasm', import.meta.url))
  const web = await createWebPolyXml(wasm)
  assert.deepEqual(web.xmlToJson('<root/>'), { root: null })
})
