import { cp, readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const directory = path.dirname(fileURLToPath(import.meta.url))
const output = path.join(directory, 'pkg')
const manifestPath = path.join(output, 'package.json')
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'))
manifest.name = '@polyxml/wasm'
manifest.main = './index.node.js'
manifest.module = './index.js'
manifest.types = './index.d.ts'
manifest.exports = {
  '.': {
    types: './index.d.ts',
    browser: './index.js',
    node: './index.node.js',
    default: './index.js',
  },
  './raw': './polyxml_wasm.js',
}
manifest.files = ['*.js', '*.wasm', '*.d.ts', 'README.md', 'LICENSE']
await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + '\n')
for (const name of ['index.js', 'index.node.js', 'stream.js', 'index.d.ts', 'README.md', 'LICENSE']) {
  await cp(path.join(directory, name), path.join(output, name))
}
