import io
import pathlib
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum

import pytest
from pydantic import BaseModel, Field

try:
    from pyxsdata.models.datatype import XmlDate, XmlDateTime, XmlDuration, XmlTime
except ImportError:

    class XmlDate:

        def __init__(self, val: str):
            self.val = val

        @classmethod
        def from_string(cls, val: str):
            return cls(val)

        def __eq__(self, other):
            return isinstance(other, XmlDate) and self.val == other.val

        def __str__(self):
            return self.val

    class XmlDateTime:

        def __init__(self, val: str):
            self.val = val

        @classmethod
        def from_string(cls, val: str):
            return cls(val)

        def __eq__(self, other):
            return isinstance(other, XmlDateTime) and self.val == other.val

        def __str__(self):
            return self.val

    class XmlTime:

        def __init__(self, val: str):
            self.val = val

        @classmethod
        def from_string(cls, val: str):
            return cls(val)

        def __eq__(self, other):
            return isinstance(other, XmlTime) and self.val == other.val

        def __str__(self):
            return self.val

    class XmlDuration:

        def __init__(self, val: str):
            self.val = val

        def __eq__(self, other):
            return isinstance(other, XmlDuration) and self.val == other.val

        def __str__(self):
            return self.val

import polyxml


def test_version():
    assert isinstance(polyxml.__version__, str)
    assert len(polyxml.__version__) > 0


@dataclass
class SimpleItem:
    id: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})
    price: float = field(metadata={"type": "Element"})
    active: bool = field(metadata={"type": "Element"})


def test_deserialize_dataclass_bytes():
    xml = b'<SimpleItem id="42"><name>Sensor</name><price>19.99</price><active>true</active></SimpleItem>'
    res = polyxml.deserialize(xml, SimpleItem)
    assert res.id == 42
    assert res.name == "Sensor"
    assert res.price == 19.99
    assert res.active is True


def test_deserialize_dataclass_str():
    xml_str = '<SimpleItem id="101"><name>Transceiver</name><price>49.50</price><active>1</active></SimpleItem>'
    res = polyxml.deserialize(xml_str, SimpleItem)
    assert res.id == 101
    assert res.name == "Transceiver"
    assert res.price == 49.50
    assert res.active is True


def test_serialize_dataclass():
    item = SimpleItem(id=77, name="Actuator", price=150.0, active=False)
    xml_bytes = polyxml.serialize(item)
    assert b'id="77"' in xml_bytes
    assert b"<name>Actuator</name>" in xml_bytes
    assert b"<price>150" in xml_bytes
    assert b"<active>false</active>" in xml_bytes


def test_serialize_with_indent():
    item = SimpleItem(id=88, name="Turbine", price=999.9, active=True)
    xml_bytes = polyxml.serialize(item, indent=2)
    assert b"  <name>Turbine</name>" in xml_bytes


@dataclass
class ChildNode:
    tag: str = field(metadata={"type": "Element"})


@dataclass
class ContainerNode:
    title: str = field(metadata={"type": "Element"})
    children: list[ChildNode] = field(
        default_factory=list, metadata={"type": "Element", "name": "child"}
    )


def test_nested_dataclasses():
    xml = """
    <ContainerNode>
        <title>Component Tree</title>
        <child><tag>Wing</tag></child>
        <child><tag>Rudder</tag></child>
    </ContainerNode>
    """
    res = polyxml.deserialize(xml, ContainerNode)
    assert res.title == "Component Tree"
    assert len(res.children) == 2
    assert res.children[0].tag == "Wing"
    assert res.children[1].tag == "Rudder"


class PydanticDevice(BaseModel):
    serial: str = Field(..., json_schema_extra={"type": "Attribute", "name": "sn"})
    model: str = Field(..., json_schema_extra={"type": "Element"})
    power: float = Field(..., json_schema_extra={"type": "Element"})


def test_pydantic_model():
    xml = (
        '<PydanticDevice sn="SN-88231"><model>AeroCore</model><power>120.5</power></PydanticDevice>'
    )
    res = polyxml.deserialize(xml, PydanticDevice)
    assert res.serial == "SN-88231"
    assert res.model == "AeroCore"
    assert res.power == 120.5


def test_invalid_xml_raises_value_error():
    with pytest.raises(ValueError):
        polyxml.deserialize(b"<UnclosedTag><name>Test</UnclosedTag", SimpleItem)


def test_iterparse_dataclass_bytes():
    xml = b"""
    <Catalog>
        <header><timestamp>12345</timestamp></header>
        <items>
            <SimpleItem id="1"><name>Widget A</name><price>10.5</price><active>true</active></SimpleItem>
            <SimpleItem id="2"><name>Widget B</name><price>20.0</price><active>false</active></SimpleItem>
            <SimpleItem id="3"><name>Widget C</name><price>30.25</price><active>1</active></SimpleItem>
        </items>
    </Catalog>
    """
    items = list(polyxml.iterparse(xml, SimpleItem))
    assert len(items) == 3
    assert items[0].id == 1 and items[0].name == "Widget A" and items[0].active is True
    assert items[1].id == 2 and items[1].name == "Widget B" and items[1].active is False
    assert items[2].id == 3 and items[2].name == "Widget C" and items[2].active is True


def test_iterparse_custom_tag_and_str():
    xml = """
    <Warehouse>
        <part id="10"><name>Gear</name><price>5.0</price><active>true</active></part>
        <part id="20"><name>Bolt</name><price>0.5</price><active>false</active></part>
    </Warehouse>
    """
    items = list(polyxml.iterparse(xml, SimpleItem, tag="part"))
    assert len(items) == 2
    assert items[0].id == 10 and items[0].name == "Gear"
    assert items[1].id == 20 and items[1].name == "Bolt"


def test_iterparse_empty_stream():
    xml = b"<EmptyCatalog></EmptyCatalog>"
    items = list(polyxml.iterparse(xml, SimpleItem))
    assert len(items) == 0


@dataclass
class OptionalItem:
    id: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})
    desc: str | None = field(default="default_desc", metadata={"type": "Element"})


def test_dataclass_optional_field_fallback():
    # XML omits <desc>, triggering kwargs fallback and preserving dataclass default value
    xml = b'<OptionalItem id="99"><name>Sensor X</name></OptionalItem>'
    res = polyxml.deserialize(xml, OptionalItem)
    assert res.id == 99
    assert res.name == "Sensor X"
    assert res.desc == "default_desc"


def test_serialize_pydantic():
    dev = PydanticDevice(serial="SN-999", model="AeroVibe", power=250.0)
    xml_bytes = polyxml.serialize(dev)
    assert b'sn="SN-999"' in xml_bytes
    assert b"<model>AeroVibe</model>" in xml_bytes
    assert b"<power>250" in xml_bytes


class Status(Enum):
    ACTIVE = "active"
    INACTIVE = "inactive"


class StatusCode(Enum):
    OK = 200
    NOT_FOUND = 404


class NamedStatus(Enum):
    FIRST = 1
    SECOND = 2


@dataclass
class RichTypesItem:
    rate: Decimal = field(metadata={"type": "Element"})
    status: Status = field(metadata={"type": "Element"})
    code: StatusCode = field(metadata={"type": "Element"})
    named: NamedStatus = field(metadata={"type": "Element"})
    date: XmlDate = field(metadata={"type": "Element"})
    datetime: XmlDateTime = field(metadata={"type": "Element"})
    time: XmlTime = field(metadata={"type": "Element"})
    duration: XmlDuration = field(metadata={"type": "Element"})
    tag: str = field(init=False, metadata={"type": "Element"})


def test_rich_types_deserialization_and_serialization():
    xml = (
        b"<RichTypesItem>"
        b"<rate>123.456789</rate>"
        b"<status>active</status>"
        b"<code>200</code>"
        b"<named>FIRST</named>"
        b"<date>2026-09-10</date>"
        b"<datetime>2026-09-10T14:30:00Z</datetime>"
        b"<time>14:30:00</time>"
        b"<duration>P1DT2H</duration>"
        b"<tag>calculated_val</tag>"
        b"</RichTypesItem>"
    )
    res = polyxml.deserialize(xml, RichTypesItem)
    assert res.rate == Decimal("123.456789")
    assert res.status == Status.ACTIVE
    assert res.code == StatusCode.OK
    assert res.named == NamedStatus.FIRST
    assert res.date == XmlDate.from_string("2026-09-10")
    assert res.datetime == XmlDateTime.from_string("2026-09-10T14:30:00Z")
    assert res.time == XmlTime.from_string("14:30:00")
    assert res.duration == XmlDuration("P1DT2H")
    assert res.tag == "calculated_val"

    # Serialize back
    serialized = polyxml.serialize(res)
    assert b"<rate>123.456789</rate>" in serialized
    assert b"<status>active</status>" in serialized
    assert b"<code>200</code>" in serialized
    assert b"<date>2026-09-10</date>" in serialized
    assert b"<tag>calculated_val</tag>" in serialized


def test_to_bytes_pathlib_and_file_str(tmp_path: pathlib.Path):
    xml_file = tmp_path / "item.xml"
    xml_file.write_bytes(
        b'<SimpleItem id="1"><name>A</name><price>1.0</price><active>1</active></SimpleItem>'
    )

    # pathlib.Path
    res1 = polyxml.deserialize(xml_file, SimpleItem)
    assert res1.id == 1

    # str file path
    res2 = polyxml.deserialize(str(xml_file), SimpleItem)
    assert res2.id == 1

    # str raw xml without '<' as first char (e.g. whitespace before, or plain xml)
    raw_str_not_file = (
        "   <SimpleItem id='2'><name>B</name><price>2.0</price><active>0</active></SimpleItem>"
    )
    res3 = polyxml.deserialize(raw_str_not_file, SimpleItem)
    assert res3.id == 2

    # str raw xml that doesn't start with '<' and is not a file
    # (e.g. invalid string) - should fail during parsing or return encoded
    with pytest.raises(ValueError):
        polyxml.deserialize("non_existent_file.xml", SimpleItem)


def test_to_bytes_streams():
    xml = b'<SimpleItem id="10"><name>StreamItem</name><price>3.5</price><active>true</active></SimpleItem>'
    # BytesIO
    res_bytes = polyxml.deserialize(io.BytesIO(xml), SimpleItem)
    assert res_bytes.id == 10

    # StringIO
    res_str = polyxml.deserialize(io.StringIO(xml.decode("utf-8")), SimpleItem)
    assert res_str.id == 10


def test_to_bytes_invalid_type():
    with pytest.raises(TypeError, match="Unsupported XML source type"):
        polyxml.deserialize(12345, SimpleItem)  # type: ignore[arg-type]
