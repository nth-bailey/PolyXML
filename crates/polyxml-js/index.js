const { existsSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let localFileExisted = false
let loadError = null

switch (platform) {
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'polyxml.win32-x64-msvc.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./polyxml.win32-x64-msvc.node')
          } else {
            nativeBinding = require('polyxml-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'polyxml.darwin-x64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./polyxml.darwin-x64.node')
          } else {
            nativeBinding = require('polyxml-darwin-x64')
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, 'polyxml.darwin-arm64.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./polyxml.darwin-arm64.node')
          } else {
            nativeBinding = require('polyxml-darwin-arm64')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  case 'linux':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, 'polyxml.linux-x64-gnu.node'))
        try {
          if (localFileExisted) {
            nativeBinding = require('./polyxml.linux-x64-gnu.node')
          } else {
            nativeBinding = require('polyxml-linux-x64-gnu')
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Linux: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  try {
    nativeBinding = require('./polyxml.node')
  } catch (e) {
    if (loadError) {
      throw loadError
    }
    throw new Error(`Failed to load native PolyXML addon for ${platform}-${arch}`)
  }
}

module.exports = nativeBinding
