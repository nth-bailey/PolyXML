import { createPolyXml } from '../../crates/polyxml-wasm/pkg/index.js'
import { XMLParser } from 'fast-xml-parser'

window.runPolyXmlBenchmark = async (tier, caseName, iterationOverride) => {
  const targets = { micro: 3 * 1024, medium: 1024 * 1024, large: 50 * 1024 * 1024 }
  if (!(tier in targets)) throw new Error(`Unknown tier ${tier}`)
  const record = '<item id="42"><name>Ada &amp; Bob</name><active>true</active></item>'
  const xml = `<items>${record.repeat(Math.ceil((targets[tier] - 13) / record.length))}</items>`
  const bytes = new TextEncoder().encode(xml).length
  const wasm = await createPolyXml()
  const fast = new XMLParser({ ignoreAttributes: false, attributeNamePrefix: '@', parseTagValue: true, parseAttributeValue: true, removeNSPrefix: true, textNodeName: 'value' })
  const cases = [
    ['polyxml object', () => wasm.xmlToJson(xml)],
    ['polyxml JSON bytes', () => wasm.xmlToJsonBytes(xml)],
    ['fast-xml-parser object', () => fast.parse(xml)],
    ['DOMParser DOM', () => new DOMParser().parseFromString(xml, 'application/xml')],
  ].filter(([name]) => !caseName || name === caseName)
  if (!cases.length) throw new Error('BENCH_CASE did not match a browser benchmark case')
  const iterations = iterationOverride ?? (tier === 'large' ? 5 : tier === 'medium' ? 15 : 100)
  if (!Number.isSafeInteger(iterations) || iterations < 1) throw new Error('BENCH_ITERATIONS must be positive')
  const results = []
  for (const [name, fn] of cases) {
    fn()
    const durations = []
    for (let i = 0; i < iterations; i++) {
      const start = performance.now()
      fn()
      durations.push(performance.now() - start)
    }
    durations.sort((a, b) => a - b)
    const medianMs = durations[Math.floor(durations.length / 2)]
    results.push({ name, medianMs, mbPerSec: bytes / 1048576 / (medianMs / 1000) })
  }
  return { runtime: navigator.userAgent, tier, bytes, iterations, note: 'DOMParser returns a DOM, so its output is not equivalent to object cases', results }
}
