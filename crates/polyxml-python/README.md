# PolyXML Python Bindings

High-performance native XML data-binding engine for Python dataclasses and Pydantic models.

Powered by `polyxml-core` written in Rust and PyO3 (`abi3-py312`).

## Features

- **⚡ Blazing Fast**: 16x faster deserialization and 38x faster serialization than standard Python tools.
- **🌊 Streaming `iterparse()`**: Parse multi-gigabyte XML files with O(1) constant memory (<5 MB RAM).
- **📦 Zero-GIL Binary Serialization**: Native `dumps_binary()` and `loads_binary()` using MessagePack (`msgspec`), up to 12.4x faster than pickle.
- **🎯 Full Type Support**: Dataclasses and Pydantic v2 models with zero boilerplate.

## Quickstart

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Item:
    id: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})

# 1. XML Deserialization
item = polyxml.deserialize(b'<Item id="1"><name>Gadget</name></Item>', Item)

# 2. XML Serialization
xml = polyxml.serialize(item, indent=2)

# 3. High-Speed Binary Serialization (Key-Value databases / IPC)
blob = polyxml.dumps_binary(item)
restored = polyxml.loads_binary(blob, Item)
```
