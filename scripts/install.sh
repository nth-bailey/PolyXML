#!/usr/bin/env bash
# ==============================================================================
# scripts/install.sh - Universal Installer for PolyXML CLI
# ==============================================================================
# Installs the pre-compiled PolyXML binary on Linux and macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/nth-bailey/PolyXML/main/scripts/install.sh | bash
#
# Environment variables:
#   POLYXML_VERSION       Specific version to install (e.g. "0.17.0", default: latest)
#   POLYXML_INSTALL_DIR   Target install directory (default: ~/.local/bin or /usr/local/bin)
# ==============================================================================
set -eo pipefail

REPO="nth-bailey/PolyXML"
BIN_NAME="polyxml"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}==> PolyXML CLI Installer${NC}"

# Detect OS
OS_UNAME="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "${OS_UNAME}" in
  linux*)  OS="unknown-linux-gnu" ;;
  darwin*) OS="apple-darwin" ;;
  msys*|cygwin*|mingw*) OS="pc-windows-msvc" ;;
  *)
    echo -e "${RED}Error: Unsupported operating system: ${OS_UNAME}${NC}" >&2
    exit 1
    ;;
esac

# Detect Architecture
ARCH_UNAME="$(uname -m)"
case "${ARCH_UNAME}" in
  x86_64|amd64)   ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo -e "${RED}Error: Unsupported CPU architecture: ${ARCH_UNAME}${NC}" >&2
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"

# Resolve target install directory
if [ -n "${POLYXML_INSTALL_DIR}" ]; then
  INSTALL_DIR="${POLYXML_INSTALL_DIR}"
elif [ "$(id -u)" -eq 0 ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="${HOME}/.local/bin"
fi

mkdir -p "${INSTALL_DIR}"

# Resolve version
if [ -z "${POLYXML_VERSION}" ]; then
  echo -e "Detecting latest PolyXML release..."
  TAG=$(curl -sSL -H "Accept: application/vnd.github+json" "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' || true)
  if [ -z "${TAG}" ] || [ "${TAG}" = "null" ]; then
    # Fallback to fetching tags
    TAG=$(curl -sSL "https://api.github.com/repos/${REPO}/tags" 2>/dev/null | grep '"name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' || true)
  fi
  if [ -z "${TAG}" ] || [ "${TAG}" = "null" ]; then
    TAG="v0.17.0"
  fi
else
  TAG="v${POLYXML_VERSION#v}"
fi

VERSION="${TAG#v}"
echo -e "Target:   ${GREEN}${TARGET}${NC}"
echo -e "Version:  ${GREEN}${TAG}${NC}"
echo -e "Location: ${GREEN}${INSTALL_DIR}/${BIN_NAME}${NC}"

ARCHIVE_NAME="polyxml-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ARCHIVE_NAME}"

TMP_DIR="$(mktemp -d -t polyxml-install-XXXXXX)"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo -e "Downloading ${BLUE}${DOWNLOAD_URL}${NC}..."
if curl -sSfL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${ARCHIVE_NAME}" 2>/dev/null; then
  tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}"
  BIN_SRC="${TMP_DIR}/${BIN_NAME}"
  if [ ! -f "${BIN_SRC}" ]; then
    BIN_SRC=$(find "${TMP_DIR}" -type f -name "${BIN_NAME}" | head -n 1)
  fi
  mv "${BIN_SRC}" "${INSTALL_DIR}/${BIN_NAME}"
else
  echo -e "${YELLOW}Notice: Pre-built binary archive '${ARCHIVE_NAME}' not found on release ${TAG}.${NC}"
  echo -e "Attempting installation via cargo..."
  if command -v cargo >/dev/null 2>&1; then
    cargo install polyxml-cli --root "${INSTALL_DIR}/.."
  else
    echo -e "${RED}Error: Neither release binary nor cargo was available.${NC}" >&2
    echo -e "Please install Rust/Cargo (https://rustup.rs) or download binaries manually from:" >&2
    echo -e "https://github.com/${REPO}/releases" >&2
    exit 1
  fi
fi

chmod +x "${INSTALL_DIR}/${BIN_NAME}"

# Verify installation
if command -v "${INSTALL_DIR}/${BIN_NAME}" >/dev/null 2>&1; then
  echo -e "${GREEN}✓ Successfully installed $("${INSTALL_DIR}/${BIN_NAME}" --version 2>/dev/null || echo "polyxml")${NC}"
fi

# Check PATH
case ":${PATH}:" in
  *:"${INSTALL_DIR}":*) ;;
  *)
    echo -e "\n${YELLOW}Warning: '${INSTALL_DIR}' is not in your PATH.${NC}"
    echo -e "Add it to your shell configuration:"
    echo -e "  ${BLUE}export PATH=\"${INSTALL_DIR}:\$PATH\"${NC}"
    ;;
esac

echo -e "\n${GREEN}Get started with:${NC}"
echo -e "  polyxml --help"
echo -e "  polyxml generate --help"
