import initWasm, {
  WasmSchema,
  json_to_xml as rawJsonToXml,
  xml_to_json as rawXmlToJson,
} from './polyxml_wasm.js'

const encoder = new TextEncoder()
const decoder = new TextDecoder('utf-8', { fatal: true })
let initialization

function bytes(value) {
  if (typeof value === 'string') return encoder.encode(value)
  if (value instanceof Uint8Array) return value
  if (value instanceof ArrayBuffer) return new Uint8Array(value)
  throw new TypeError('Expected a string, Uint8Array, or ArrayBuffer')
}

function jsonBytes(value) {
  if (typeof value === 'string' || value instanceof Uint8Array || value instanceof ArrayBuffer) {
    return bytes(value)
  }
  return encoder.encode(JSON.stringify(value))
}

function wrapSchema(schema) {
  return {
    xmlToJsonBytes(xml) { return schema.xml_to_json(bytes(xml)) },
    xmlToJson(xml) { return JSON.parse(decoder.decode(schema.xml_to_json(bytes(xml)))) },
    jsonToXmlBytes(json) { return schema.json_to_xml(jsonBytes(json)) },
    jsonToXml(json) { return decoder.decode(schema.json_to_xml(jsonBytes(json))) },
    free() { schema.free() },
  }
}

/** Initialize the Wasm module and return the XML/JSON API. */
export async function createPolyXml(source = new URL('./polyxml_wasm_bg.wasm', import.meta.url)) {
  initialization ??= initWasm({ module_or_path: source }).catch((error) => {
    initialization = undefined
    throw error
  })
  await initialization
  return {
    xmlToJsonBytes(xml) { return rawXmlToJson(bytes(xml)) },
    xmlToJson(xml) { return JSON.parse(decoder.decode(rawXmlToJson(bytes(xml)))) },
    jsonToXmlBytes(json, rootName) { return rawJsonToXml(jsonBytes(json), rootName) },
    jsonToXml(json, rootName) { return decoder.decode(rawJsonToXml(jsonBytes(json), rootName)) },
    schemaFromXsd(xsd, rootName) { return wrapSchema(new WasmSchema(xsd, rootName)) },
  }
}
