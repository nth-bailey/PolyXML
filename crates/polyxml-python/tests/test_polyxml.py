from dataclasses import dataclass, field

import polyxml
import pytest
from pydantic import BaseModel, Field


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
    xml = '<PydanticDevice sn="SN-88231"><model>AeroCore</model><power>120.5</power></PydanticDevice>'
    res = polyxml.deserialize(xml, PydanticDevice)
    assert res.serial == "SN-88231"
    assert res.model == "AeroCore"
    assert res.power == 120.5


def test_invalid_xml_raises_value_error():
    with pytest.raises(ValueError):
        polyxml.deserialize(b"<UnclosedTag><name>Test</UnclosedTag", SimpleItem)
