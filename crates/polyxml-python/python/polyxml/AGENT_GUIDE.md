# PolyXML AI Agent Guide

Quick-reference instructions for AI coding assistants (Claude, Cursor, Copilot, Antigravity, etc.) using `polyxml` in Python projects.

---

## 1. Overview & Capabilities

`polyxml` is an ultra-fast, native XML data-binding engine written in pure Rust (using `quick-xml` and `lexical-core`) with stable ABI (`abi3-py312`) Python bindings. It provides bidirectional deserialization and serialization for Python 3.12+ `dataclasses` and Pydantic v2 models.

---

## 2. Common Recipes

### A. Deserializing XML (Strings, Bytes, or Files)
```python
from dataclasses import dataclass, field
import pathlib
import polyxml


@dataclass
class Item:
    id: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})
    price: float = field(metadata={"type": "Element"})


# 1. From raw XML string:
item = polyxml.deserialize('<Item id="1"><name>Bolt</name><price>0.99</price></Item>', Item)

# 2. From raw bytes:
item = polyxml.deserialize(b"<Item ...>...</Item>", Item)

# 3. From pathlib.Path:
item = polyxml.deserialize(pathlib.Path("item.xml"), Item)
```

### B. Streaming Massive XML Files with O(1) Constant Memory
For large datasets (hundreds of MBs or GBs such as transit timetables or feeds), do **NOT** load the entire XML tree at once. Use `iterparse`:

```python
import polyxml
from my_models import VehicleJourney

# Streams elements one-by-one with zero intermediate DOM allocations
for journey in polyxml.iterparse("large_transit_dump.xml", VehicleJourney, tag="VehicleJourney"):
    process_journey(journey)
```

### C. Pydantic v2 Models
PolyXML natively deserializes into Pydantic v2 `BaseModel` classes with the same API:

```python
from pydantic import BaseModel, Field
import polyxml


class StopPoint(BaseModel):
    id: str = Field(json_schema_extra={"type": "Attribute"})
    name: str


stop = polyxml.deserialize(
    '<StopPoint id="SP1"><name>Central Station</name></StopPoint>', StopPoint
)
```

### D. Serializing to XML
```python
import polyxml

# Serialize model to UTF-8 XML bytes
xml_bytes = polyxml.serialize(item, indent=2)

# If string is required:
xml_string = xml_bytes.decode("utf-8")
```

---

## 3. Common Pitfalls & Quick Fixes

- **Pitfall**: Attempting `polyxml.XmlParser()`  
  **Fix**: PolyXML uses module-level functional entrypoints: `polyxml.deserialize(...)` and `polyxml.iterparse(...)`.
- **Pitfall**: Passing file handles or paths without reading  
  **Fix**: `polyxml.deserialize(...)` accepts strings, byte buffers, `pathlib.Path`, or `IO` streams directly.
- **Pitfall**: High memory usage on massive XML files  
  **Fix**: Switch from `polyxml.deserialize` to `polyxml.iterparse(source, ModelClass, tag="TagName")`.
