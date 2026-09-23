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

test('parseStream emits records across tiny byte chunks', async () => {
  const xml = '<rows xmlns:x="urn:test"><x:item id="1">A &amp; B</x:item><x:item><![CDATA[<two>]]></x:item><x:item id="3"/></rows>'
  const encoded = new TextEncoder().encode(xml)
  const stream = new ReadableStream({
    start(controller) {
      for (const byte of encoded) controller.enqueue(Uint8Array.of(byte))
      controller.close()
    },
  })
  const records = []
  for await (const record of polyxml.parseStream(stream)) records.push(record)
  assert.deepEqual(records, [{ item: { '@id': 1, value: 'A & B' } }, { item: '<two>' }, { item: { '@id': 3 } }])
})

test('parseStream enforces record limits and rejects incomplete input', async () => {
  async function* chunks(...parts) { for (const part of parts) yield part }
  await assert.rejects(async () => {
    for await (const _ of polyxml.parseStream(chunks('<rows><item>long</item></rows>'), { maxRecordBytes: 4 })) {}
  }, /exceeds maxRecordBytes/)
  await assert.rejects(async () => {
    for await (const _ of polyxml.parseStream(chunks('<rows><item>unfinished'))) {}
  }, /Incomplete XML document/)
})

test('parseStream handles split UTF-8, quoted tag delimiters, and early records', async () => {
  const xml = '<rows><item label="a>b">café</item><item>next</item></rows>'
  const input = new TextEncoder().encode(xml)
  const boundary = input.indexOf(0xc3)
  const nextItem = new TextEncoder().encode(xml.slice(0, xml.indexOf('<item>next'))).length
  let continued = false
  async function* chunks() {
    yield input.slice(0, boundary + 1)
    yield input.slice(boundary + 1, nextItem)
    continued = true
    yield input.slice(nextItem)
  }
  const iterator = polyxml.parseStream(chunks())[Symbol.asyncIterator]()
  assert.deepEqual((await iterator.next()).value, { item: { '@label': 'a>b', value: 'café' } })
  assert.equal(continued, false)
  assert.deepEqual((await iterator.next()).value, { item: 'next' })
  assert.equal((await iterator.next()).done, true)
})
