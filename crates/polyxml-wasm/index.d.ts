export type Bytes = string | Uint8Array | ArrayBuffer

export interface PolyXmlSchema {
  xmlToJsonBytes(xml: Bytes): Uint8Array
  xmlToJson(xml: Bytes): unknown
  jsonToXmlBytes(json: unknown): Uint8Array
  jsonToXml(json: unknown): string
  free(): void
}

export interface PolyXml {
  xmlToJsonBytes(xml: Bytes): Uint8Array
  xmlToJson(xml: Bytes): unknown
  jsonToXmlBytes(json: unknown, rootName?: string): Uint8Array
  jsonToXml(json: unknown, rootName?: string): string
  schemaFromXsd(xsd: string, rootName?: string): PolyXmlSchema
}

export function createPolyXml(source?: BufferSource | WebAssembly.Module | Response | URL): Promise<PolyXml>
