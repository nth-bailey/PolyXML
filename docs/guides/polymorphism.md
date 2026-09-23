# Polymorphic Types & `xsi:type` Dispatch

XSD type hierarchies built from `<xs:complexContent><xs:extension>` (and
`xsi:type` wire selectors) are dispatched by PolyXML's **dynamic runtime** —
the shared Rust engine behind the native API, `polyxml-python`, and (as it
gains nested schemas) `polyxml-js`. This page documents the strategy, its
round-trip behavior, and the known limitations of the static codegen paths.

---

## 🧭 Strategy: a type registry on `ModelSchema`

Every runtime `ModelSchema` carries:

- `is_abstract` — the source schema marked the type `abstract="true"`.
- a **`variants` registry** — the concrete derivations eligible for
  dispatch, matched by its **namespace and local name** (e.g.
  `urn:veh:Car`).

The registry is populated from whichever schema source built the model:

| Source | How variants are discovered |
|---|---|
| **XSD** (`ModelSchema::from_ir`) | All *transitive* `xs:extension` derivations of the type are built recursively from the `SchemaIR`. Derived schemas **flatten the full base-chain content model**: inherited fields precede the type's own fields. |
| **Python classes** (PyO3) | A `__subclasses__()` walk of the dataclass/Pydantic hierarchy, keyed by `Meta.name` (falling back to the class name). Generated Python already emits real class inheritance (`class Car(Vehicle):`), so inherited fields and metadata flow automatically. |

`xsi:type` is resolved as a QName using namespace declarations in scope at
the element. The attribute itself must be bound to the XML Schema Instance
namespace. A type with the same local name in a different namespace does not
select a registered variant.

---

## 📥 Parse behavior

Dispatch is applied at **every frame creation point**: the document root
(`Start` and `Empty`), nested elements, list items, and streamed
`iterparse` records.

- `xsi:type` names a registered derivation → the element is parsed with the
  concrete schema, so **base fields and concrete fields both survive**.
- An abstract declaration without `xsi:type` is rejected.
- `xsi:type` names an unknown type on an **abstract** base → a loud
  `ValueError` listing the known derivations:
  ```
  xsi:type="t:Plane" does not match any known derivation of 'Vehicle' (known: Car, Truck)
  ```
- An abstract type with **no registered derivations** fails with an
  explicit escape-hatch message telling you to deserialize the concrete
  type directly.
- Everything else is **tolerant**: non-abstract types without derivations
  never scan for `xsi:type` at all (zero-overhead fast path), so a plain
  content attribute literally named `type` is never mistaken for a
  selector; unknown selectors on non-abstract types parse as declared.

---

## 📤 Serialize behavior

When a value is a record built from a **registered variant** of the
declared schema, the serializer writes the concrete type's fields and
re-emits the selector (`xsi:type="t:Car"`), declaring the XML Schema
Instance namespace at the root automatically.

Python has two entry points:

```python
# Serialize against the concrete type (default): the element is <Car>.
polyxml.serialize(car)

# Serialize against the declared base: the element stays <Vehicle ...>
# and xsi:type="Car" is re-emitted — a faithful wire round trip.
polyxml.serialize(car, target_type=Vehicle)
```

### Round trip

```python
from dataclasses import dataclass, field
import polyxml

@dataclass
class Vehicle:
    class Meta:
        name = "Vehicle"
        abstract = True
    id: str = field(metadata={"type": "Element", "name": "id"})

@dataclass
class Car(Vehicle):
    class Meta:
        name = "Car"
    doors: int = field(metadata={"type": "Element", "name": "doors"})

@dataclass
class Fleet:
    class Meta:
        name = "Fleet"
    vehicle: Vehicle = field(metadata={"type": "Element", "name": "vehicle"})

xml = b'<Fleet xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><vehicle xsi:type="Car"><id>V1</id><doors>4</doors></vehicle></Fleet>'

fleet = polyxml.deserialize(xml, Fleet)     # fleet.vehicle is a Car
assert fleet.vehicle.doors == 4             # concrete field preserved
out = polyxml.serialize(fleet)              # b'... xsi:type="Car" ...'
assert polyxml.deserialize(out, Fleet) == fleet
```

The same registry is reachable from pure Rust:

```rust
let ir = polyxml::schema_parser::XsdParser::new().parse_str(xsd)?;
let schema = polyxml::schema::ModelSchema::from_ir(&ir, Some("Root"))?;
let value = polyxml::deserialize(xml, schema.clone())?;   // dispatched Record
let out = polyxml::serialize_with_options("Root", &value, &schema, None, Some(true), None)?;
```

---

## ⚠️ Known limitations & escape hatches

1. **Generated static codecs do not dispatch.** The compiled serde/quick-xml
   codecs (Rust, Java, …) bind each element to one declared type; the Java
   codec rejects `xsi:type` outright. *Escape hatch:* deserialize the
   concrete type directly, or route polymorphic payloads through the
   dynamic runtime (`polyxml::deserialize`, `polyxml.deserialize`).
2. **`polyxml-js` is scalar-flat today.** Its schema binding exposes only
   scalar fields, so complex-typed (and therefore polymorphic) fields are
   unreachable from JS yet — the dispatch itself lives in the shared core,
   so no new core dispatch algorithm is needed when the JS binding gains
   nested schemas.
3. **JSON transcoding drops the selector.** `xml_to_json` keeps every
   concrete field (it iterates the record's own schema), but JSON carries
   no `xsi:type` marker, so JSON → XML cannot recover the wire selector.
4. **Discovery timing (Python).** Cached schemas refresh their variant
   registries when used, including nested fields, so subclasses imported
   after the first deserialize can be discovered on a later call.
5. **Unknown derivations on abstract bases fail loudly.** An `xsi:type`
   naming an unregistered type on an abstract base is an error. Import the
   module defining the subclass or deserialize with the concrete type.
   Non-abstract declarations without a matching registered variant parse as
   their declared type.

Regression coverage lives in
`crates/polyxml-core/tests/test_xsi_type.rs` (Rust, 7 cases) and
`crates/polyxml-python/tests/test_xsi_type.py` (Python, 11 cases).
