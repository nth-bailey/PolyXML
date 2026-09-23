import { chromium } from 'playwright'
import { createServer } from 'vite'
import { resolve } from 'node:path'

const tier = process.argv[2] ?? 'micro'
const server = await createServer({ root: import.meta.dirname, server: { port: 0, host: '127.0.0.1', fs: { allow: [resolve(import.meta.dirname, '../..')] } } })
let browser
try {
  await server.listen()
  const address = server.httpServer.address()
  browser = await chromium.launch({ headless: true, executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE })
  const page = await browser.newPage()
  await page.goto(`http://127.0.0.1:${address.port}/browser.html`)
  await page.waitForFunction(() => typeof window.runPolyXmlBenchmark === 'function')
  const selection = { tier, caseName: process.env.BENCH_CASE, iterations: process.env.BENCH_ITERATIONS ? Number(process.env.BENCH_ITERATIONS) : undefined }
  console.log(JSON.stringify(await page.evaluate(({ tier, caseName, iterations }) => window.runPolyXmlBenchmark(tier, caseName, iterations), selection), null, 2))
} finally {
  await browser?.close()
  await server.close()
}
