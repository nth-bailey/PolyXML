---
title: Python Guide (Dataclasses & Pydantic v2)
description: Ultra-fast XML deserialization and serialization into Python dataclasses and Pydantic v2 models.
---

# Python Guide

PolyXML delivers high-throughput native XML data binding for Python `>=3.12`.

## Installation

```bash
pip install polyxml
```

## 1. Python Dataclasses

PolyXML supports standard Python `dataclasses` using `metadata`:

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Aircraft:
    tail_number: str = field(metadata={"type": "Attribute", "name": "tail"})
    heading: float = field(metadata={"type": "Element"})
    in_flight: bool = field(metadata={"type": "Element"})

xml = b'<Aircraft tail="N1024A"><heading>270.5</heading><in_flight>true</in_flight></Aircraft>'

aircraft = polyxml.deserialize(xml, Aircraft)
print(aircraft.tail_number)  # "N1024A"

# Serialize back to XML bytes
xml_out = polyxml.serialize(aircraft, indent=2)
```

## 2. Pydantic v2 Models

PolyXML supports Pydantic v2 `BaseModel` via `json_schema_extra`:

```python
from pydantic import BaseModel, Field
import polyxml

class SystemStatus(BaseModel):
    code: int = Field(..., json_schema_extra={"type": "Attribute"})
    message: str = Field(..., json_schema_extra={"type": "Element"})

status = polyxml.deserialize(b'<SystemStatus code="200"><message>OK</message></SystemStatus>', SystemStatus)
print(status.message)  # "OK"
```
