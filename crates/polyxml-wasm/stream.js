const encoder = new TextEncoder()

function tagEnd(source, start) {
  if (source.startsWith('<!--', start)) {
    const end = source.indexOf('-->', start + 4)
    return end < 0 ? -1 : end + 3
  }
  if (source.startsWith('<![CDATA[', start)) {
    const end = source.indexOf(']]>', start + 9)
    return end < 0 ? -1 : end + 3
  }
  if (source.startsWith('<?', start)) {
    const end = source.indexOf('?>', start + 2)
    return end < 0 ? -1 : end + 2
  }
  let quote = ''
  for (let i = start + 1; i < source.length; i++) {
    const char = source[i]
    if (quote) {
      if (char === quote) quote = ''
    } else if (char === '"' || char === "'") {
      quote = char
    } else if (char === '>') {
      return i + 1
    }
  }
  return -1
}

/** Yield each direct child of an XML document root as soon as it is complete.
 * Memory use is bounded by the largest child and root tag, subject to maxRecordBytes.
 */
export async function* parseStream(api, source, { maxRecordBytes = 16 * 1024 * 1024 } = {}) {
  if (!Number.isSafeInteger(maxRecordBytes) || maxRecordBytes < 1) {
    throw new RangeError('maxRecordBytes must be a positive safe integer')
  }
  const input = source?.getReader ? readableChunks(source) : source
  if (!input?.[Symbol.asyncIterator]) throw new TypeError('Expected a ReadableStream or async iterable')
  const utf8 = new TextDecoder('utf-8', { fatal: true })
  let buffer = ''
  let cursor = 0
  let root = ''
  let rootName = ''
  let childStart = -1
  let childName = ''
  const names = []
  let done = false

  for await (const chunk of input) {
    if (typeof chunk !== 'string' && !(chunk instanceof Uint8Array) && !(chunk instanceof ArrayBuffer)) {
      throw new TypeError('Stream chunks must be strings, Uint8Array, or ArrayBuffer')
    }
    buffer += typeof chunk === 'string' ? chunk : utf8.decode(chunk, { stream: true })
    while (true) {
      const start = buffer.indexOf('<', cursor)
      if (start < 0) break
      if (childStart < 0 && buffer.slice(cursor, start).trim()) {
        throw new Error('Text outside a record is not supported')
      }
      if (buffer.length - start < 2) break
      const end = tagEnd(buffer, start)
      if (end < 0) break
      const tag = buffer.slice(start, end)
      cursor = end
      if (tag.startsWith('<!--') || tag.startsWith('<?')) continue
      if (tag.startsWith('<![CDATA[')) {
        if (childStart < 0) throw new Error('CDATA outside a record is not supported')
        continue
      }
      if (tag.startsWith('<!')) throw new Error('DOCTYPE and declarations are not supported by parseStream')
      const closing = tag.startsWith('</')
      const name = /^<\/?([^\s/>]+)/.exec(tag)?.[1]
      if (!name) throw new Error('Invalid XML tag')
      const empty = !closing && /\/\s*>$/.test(tag)
      if (!root) {
        if (closing || empty) throw new Error('Expected a nonempty document root')
        root = tag
        rootName = name
      } else if (done) {
        throw new Error('Content after document root')
      } else if (closing && names.length === 0) {
        if (name !== rootName) throw new Error('Mismatched document root')
        done = true
      } else if (closing) {
        if (names.pop() !== name) throw new Error('Mismatched XML end tag')
      } else {
        if (names.length === 0) {
          childStart = start
          childName = name.split(':').pop()
        }
        if (!empty) names.push(name)
      }
      if (childStart >= 0 && names.length === 0) {
        const child = buffer.slice(childStart, end)
        if (encoder.encode(child).length > maxRecordBytes) throw new RangeError('XML record exceeds maxRecordBytes')
        const document = root + child + `</${rootName}>`
        const result = api.xmlToJson(document)
        yield { [childName]: result[rootName][childName] }
        buffer = buffer.slice(end)
        cursor = 0
        childStart = -1
      }
    }
    if (childStart >= 0 && buffer.length - childStart > maxRecordBytes) {
      throw new RangeError('XML record exceeds maxRecordBytes')
    }
    if (!root && buffer.length > maxRecordBytes) {
      throw new RangeError('XML root tag exceeds maxRecordBytes')
    }
    if (childStart < 0 && !buffer.trim()) {
      buffer = ''
      cursor = 0
    }
    if (childStart < 0 && cursor > 0) {
      buffer = buffer.slice(cursor)
      cursor = 0
    }
  }
  buffer += utf8.decode()
  if (!done || buffer.trim()) throw new Error('Incomplete XML document')
}

async function* readableChunks(stream) {
  const reader = stream.getReader()
  try {
    while (true) {
      const { done, value } = await reader.read()
      if (done) return
      yield value
    }
  } finally {
    reader.releaseLock()
  }
}
