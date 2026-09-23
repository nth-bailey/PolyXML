import { performance, PerformanceObserver } from 'node:perf_hooks'
import { setImmediate } from 'node:timers/promises'
import { createRequire } from 'node:module'
import { createPolyXml } from '../../crates/polyxml-wasm/pkg/index.node.js'
import { XMLParser } from 'fast-xml-parser'

const require = createRequire(import.meta.url)
const xml2js = require('xml2js')
const tier = process.argv[2] ?? 'micro'
const targets = { micro: 3 * 1024, medium: 1024 * 1024, large: 50 * 1024 * 1024 }
if (!(tier in targets)) throw new Error(`Unknown tier ${tier}`)

const record = '<item id="42"><name>Ada &amp; Bob</name><active>true</active></item>'
const count = Math.max(1, Math.ceil((targets[tier] - 13) / record.length))
const xml = `<items>${record.repeat(count)}</items>`
const bytes = Buffer.byteLength(xml)
const wasm = await createPolyXml()
const fast = new XMLParser({ ignoreAttributes: false, attributeNamePrefix: '@', parseTagValue: true, parseAttributeValue: true, removeNSPrefix: true, textNodeName: 'value' })
const cases = [
  ['polyxml object', () => wasm.xmlToJson(xml)],
  ['polyxml JSON bytes', () => wasm.xmlToJsonBytes(xml)],
  ['fast-xml-parser object', () => fast.parse(xml)],
  ['xml2js object', () => xml2js.parseStringPromise(xml, { explicitArray: false })],
].filter(([name]) => !process.env.BENCH_CASE || name === process.env.BENCH_CASE)
if (!cases.length) throw new Error('BENCH_CASE did not match a benchmark case')

const gcEvents = []
let observer
try {
  observer = new PerformanceObserver(list => {
    for (const entry of list.getEntries()) gcEvents.push(entry.duration)
  })
  observer.observe({ entryTypes: ['gc'] })
} catch { /* Bun does not expose V8 GC entries. */ }

const iterations = Number(process.env.BENCH_ITERATIONS ?? (tier === 'large' ? 5 : tier === 'medium' ? 15 : 100))
if (!Number.isSafeInteger(iterations) || iterations < 1) throw new Error('BENCH_ITERATIONS must be positive')
const results = []
for (const [name, fn] of cases) {
  await fn()
  const durations = []
  const heapDeltas = []
  let gcEventCount = 0
  let gcMaxPauseMs = 0
  for (let i = 0; i < iterations; i++) {
    globalThis.gc?.()
    await setImmediate()
    const gcStart = gcEvents.length
    const beforeHeap = process.memoryUsage().heapUsed
    const start = performance.now()
    await fn()
    durations.push(performance.now() - start)
    heapDeltas.push(process.memoryUsage().heapUsed - beforeHeap)
    await setImmediate()
    const caseGc = gcEvents.slice(gcStart)
    gcEventCount += caseGc.length
    gcMaxPauseMs = Math.max(gcMaxPauseMs, ...caseGc)
  }
  durations.sort((a, b) => a - b)
  results.push({ name, medianMs: durations[Math.floor(durations.length / 2)], mbPerSec: bytes / 1048576 / (durations[Math.floor(durations.length / 2)] / 1000), medianHeapDeltaBytes: heapDeltas.sort((a, b) => a - b)[Math.floor(heapDeltas.length / 2)], gcEventCount, gcMaxPauseMs })
}
observer?.disconnect()
console.log(JSON.stringify({ runtime: typeof Bun === 'undefined' ? `Node ${process.version}` : `Bun ${Bun.version}`, tier, bytes, iterations, note: 'GC entries are counted only during the parse, where the runtime exposes them', results }, null, 2))
