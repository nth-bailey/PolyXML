---
title: Python Guide (Dataclasses & Pydantic v2)
description: High-throughput XML serialization and deserialization into Python dataclasses and Pydantic v2 models with zero boilerplate.
---

# Python Guide: Dataclasses & Pydantic v2

PolyXML is the fastest typed XML data-binding engine for Python `>=3.12`. Compiled natively in Rust via PyO3 with `abi3` stability, it replaces slow pure-Python parsers like `xsdata` with up to **16.8x faster deserialization** and **38.5x faster serialization**.

---

## 📦 Installation

```bash
pip install polyxml
```

PolyXML distributes pre-compiled binary wheels with forward compatibility across **Python 3.12, 3.13, and 3.14**.

---

## 1. Standard Python Dataclasses

PolyXML parses XML directly into standard Python `@dataclass` models without requiring base class inheritance. You control XML mapping using standard `dataclasses.field(metadata={...})`.

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Aircraft:
    # Mapped as an XML attribute (<Aircraft tail="N1024A">)
    tail_number: str = field(metadata={"type": "Attribute", "name": "tail"})
    
    # Mapped as a child element (<Aircraft><heading>270.5</heading></Aircraft>)
    heading: float = field(metadata={"type": "Element"})
    
    # Defaults to Element if metadata is omitted
    in_flight: bool = True
    speed_knots: int = 250

xml = b"""
<Aircraft tail="N1024A">
    <heading>270.5</heading>
    <in_flight>true</in_flight>
    <speed_knots>320</speed_knots>
</Aircraft>
"""

# 1. Deserialize XML bytes directly into the dataclass
plane = polyxml.deserialize(xml, Aircraft)
print(f"Tail: {plane.tail_number}, Heading: {plane.heading}°, Speed: {plane.speed_knots}kt")
# Output: Tail: N1024A, Heading: 270.5°, Speed: 320kt

# 2. Serialize dataclass back to formatted XML
output = polyxml.serialize(plane, indent=2)
print(output.decode("utf-8"))
```

---

## 2. Nested Dataclasses & Repeated Collections (`list[T]`)

Hierarchical XML structures map naturally to nested dataclasses and `list[T]` collections.

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Waypoint:
    name: str = field(metadata={"type": "Attribute"})
    altitude_ft: float = field(metadata={"type": "Element"})

@dataclass
class FlightPlan:
    callsign: str = field(metadata={"type": "Attribute"})
    origin: str = field(metadata={"type": "Element"})
    destination: str = field(metadata={"type": "Element"})
    waypoints: list[Waypoint] = field(metadata={"type": "Element", "name": "Waypoint"}, default_factory=list)

xml = b"""
<FlightPlan callsign="UAL412">
    <origin>KSFO</origin>
    <destination>KORD</destination>
    <Waypoint name="MVA"><altitude_ft>18000.0</altitude_ft></Waypoint>
    <Waypoint name="OBH"><altitude_ft>33000.0</altitude_ft></Waypoint>
    <Waypoint name="IOW"><altitude_ft>24000.0</altitude_ft></Waypoint>
</FlightPlan>
"""

plan = polyxml.deserialize(xml, FlightPlan)
print(f"Flight {plan.callsign}: {plan.origin} -> {plan.destination}")
for wp in plan.waypoints:
    print(f"  - Waypoint {wp.name}: {wp.altitude_ft} ft")
```

---

## 3. Pydantic v2 Models & Runtime Validation

PolyXML fully supports Pydantic v2 `BaseModel` classes using `json_schema_extra` for XML metadata. PolyXML populates the Pydantic model directly, preserving all Pydantic validators, coercions, and schema generation.

```python
from pydantic import BaseModel, Field, field_validator
import polyxml

class SystemMetric(BaseModel):
    sensor_id: int = Field(..., json_schema_extra={"type": "Attribute", "name": "id"})
    label: str = Field(..., json_schema_extra={"type": "Element"})
    cpu_percent: float = Field(..., json_schema_extra={"type": "Element"})

    @field_validator("cpu_percent")
    @classmethod
    def validate_cpu(cls, v: float) -> float:
        if not (0.0 <= v <= 100.0):
            raise ValueError(f"CPU usage must be between 0 and 100, got {v}")
        return v

xml = b'<SystemMetric id="1"><label>EdgeNode-01</label><cpu_percent>42.8</cpu_percent></SystemMetric>'

metric = polyxml.deserialize(xml, SystemMetric)
print(f"Node {metric.sensor_id} ({metric.label}): {metric.cpu_percent}% CPU")

# Serialization preserves Pydantic models
serialized = polyxml.serialize(metric, indent=2)
print(serialized.decode("utf-8"))
```

---

## 4. Constant-Memory Streaming with `polyxml.iterparse`

For large documents (hundreds of megabytes to gigabytes), parsing the entire document at once can cause out-of-memory errors. `polyxml.iterparse` yields typed objects one by one in constant $O(1)$ memory:

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class LogEntry:
    id: int = field(metadata={"type": "Attribute"})
    level: str = field(metadata={"type": "Element"})
    message: str = field(metadata={"type": "Element"})

# Large streaming XML document
large_xml = b"""
<ServerLog>
    <LogEntry id="1"><level>INFO</level><message>Server initialized</message></LogEntry>
    <LogEntry id="2"><level>WARN</level><message>High latency detected</message></LogEntry>
    <LogEntry id="3"><level>ERROR</level><message>Database timeout</message></LogEntry>
</ServerLog>
"""

# Stream records matching tag "LogEntry" with O(1) memory overhead
error_count = 0
for entry in polyxml.iterparse(large_xml, LogEntry, tag="LogEntry"):
    if entry.level == "ERROR":
        print(f"[ALERT] Entry #{entry.id}: {entry.message}")
        error_count += 1

print(f"Processing complete. Errors found: {error_count}")
```

---

## 5. Serialization & Pretty-Printing

PolyXML provides high-throughput serialization with configurable formatting:

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Config:
    env: str = field(metadata={"type": "Attribute"})
    debug: bool = field(metadata={"type": "Element"})
    max_retries: int = field(metadata={"type": "Element"})

cfg = Config(env="production", debug=False, max_retries=5)

# Compact output (indent=None or indent=0)
compact = polyxml.serialize(cfg)
print("Compact:", compact.decode("utf-8"))

# Pretty-printed with 2-space indentation
pretty = polyxml.serialize(cfg, indent=2)
print("Pretty:\n" + pretty.decode("utf-8"))
```

---

## 6. Migrating from `xsdata` and `ElementTree`

Switching from `xsdata` or standard `xml.etree.ElementTree` to PolyXML is drop-in and delivers immediate order-of-magnitude speedups:

### Comparison: Deserialization

=== "PolyXML (Native Rust)"
    ```python
    import polyxml
    # 13.48 ms for 10,000 items (16.8x faster)
    result = polyxml.deserialize(xml_bytes, Catalog)
    ```

=== "xsdata (Pure Python)"
    ```python
    from xsdata.formats.dataclass.parsers import XmlParser
    # 226.94 ms for 10,000 items
    parser = XmlParser()
    result = parser.from_bytes(xml_bytes, Catalog)
    ```

=== "xml.etree.ElementTree"
    ```python
    import xml.etree.ElementTree as ET
    # 12.28 ms for raw untyped DOM + ~18 ms manual loop = ~30 ms
    root = ET.fromstring(xml_bytes)
    items = [
        CatalogItem(
            id=int(el.attrib["id"]),
            name=el.findtext("name"),
            price=float(el.findtext("price"))
        )
        for el in root.findall("item")
    ]
    ```

### Key Migration Benefits:
- **No Parser Contexts Needed**: `polyxml.deserialize` is a pure function.
- **Fast Constructor Calling**: PolyXML uses positional tuples `cls(*args)` internally, eliminating dictionary allocations.
- **Zero Schema Compilation**: Works directly with standard `@dataclass` and Pydantic models.

---

## 7. Zero-GIL Binary Serialization (`dumps_binary` & `loads_binary`)

When caching parsed models in transactional key-value stores (such as `libmdbx`, `LMDB`, or `Redis`) or passing objects across multiprocessing workers, re-serializing to XML or using Python's standard `pickle`/`cloudpickle` creates severe CPU and GIL bottlenecks.

`polyxml.dumps_binary` and `polyxml.loads_binary` provide high-throughput MessagePack binary serialization:

```bash
pip install "polyxml[msgpack]"
```

### Usage Example

```python
from dataclasses import dataclass
from decimal import Decimal
from enum import Enum
import pathlib
from xml.etree.ElementTree import QName
from pyxsdata.models.datatype import XmlDate, XmlDateTime, XmlDuration, XmlTime
import polyxml

class RouteMode(str, Enum):
    BUS = "bus"
    RAIL = "rail"

@dataclass
class ServiceJourney:
    id: str
    mode: RouteMode
    fare: Decimal
    service_date: XmlDate
    departure: XmlTime
    timestamp: XmlDateTime
    duration: XmlDuration
    schema_type: QName
    source_file: pathlib.Path

journey = ServiceJourney(
    id="SJ-402",
    mode=RouteMode.BUS,
    fare=Decimal("4.50"),
    service_date=XmlDate(2026, 9, 12),
    departure=XmlTime(14, 30, 0),
    timestamp=XmlDateTime(2026, 9, 12, 14, 30, 0),
    duration=XmlDuration("PT45M"),
    schema_type=QName("http://www.netex.org.uk/netex", "ServiceJourney"),
    source_file=pathlib.Path("/data/netex/timetable.xml"),
)

# 1. Direct typed binary encoding & decoding (maximum speed: 200,000+ ops/s)
blob = polyxml.dumps_binary(journey)
restored = polyxml.loads_binary(blob, ServiceJourney)
assert restored == journey

# 2. Self-describing tagged envelopes (tag_class=True)
# Encodes (module:qualname, payload) so untyped loads_binary() reconstructs the exact class
tagged_blob = polyxml.dumps_binary(journey, tag_class=True)
dynamic_obj = polyxml.loads_binary(tagged_blob)
assert isinstance(dynamic_obj, ServiceJourney)
assert dynamic_obj.fare == Decimal("4.50")
assert dynamic_obj.duration == XmlDuration("PT45M")

# 3. Full Pydantic v2 support
from pydantic import BaseModel

class UserProfile(BaseModel):
    username: str
    balance: Decimal

user = UserProfile(username="alex", balance=Decimal("125.75"))
pydantic_blob = polyxml.dumps_binary(user)
restored_user = polyxml.loads_binary(pydantic_blob, UserProfile)
assert restored_user.balance == Decimal("125.75")
```

### Key Performance & Architecture Advantages:
- **7.9x Faster than Pickle**: Slashes serialization latency from 48.5 μs to 6.1 μs per entity.
- **53.9% Smaller Storage Footprint**: Drops entity storage from 547 bytes to 252 bytes in transactional databases like `libmdbx`.
- **3.2x Faster MDBX Database Writes**: 58,781 ops/sec transactional throughput compared to 18,287 ops/sec with legacy `cloudpickle + lz4`.
- **Universal Schema Leaf Support**: Out-of-the-box lossless handling of `XmlDate`, `XmlDateTime`, `XmlDuration`, `XmlTime`, `Decimal`, `QName`, `Enum`, `Path`, `UserString`, and any object implementing `.from_string()`.
- **Zero Schema Compilation**: Introspects dataclasses dynamically in C with zero manual boilerplate or per-class serializer generation.


