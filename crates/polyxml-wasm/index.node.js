import { readFile } from 'node:fs/promises'
import { createPolyXml as createPolyXmlWeb } from './index.js'

/** Node.js and Bun entry point; loads the local Wasm file automatically. */
export async function createPolyXml(source) {
  const moduleSource = source ?? await readFile(new URL('./polyxml_wasm_bg.wasm', import.meta.url))
  return createPolyXmlWeb(moduleSource)
}
