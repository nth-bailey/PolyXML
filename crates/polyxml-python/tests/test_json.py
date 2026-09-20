import io
import pathlib
from dataclasses import dataclass, field
from decimal import Decimal
from enum import Enum

import pytest
from pydantic import BaseModel, Field

import polyxml
from polyxml.compat.xsdata import (
    JsonParser as CompatJsonParser,
)
from polyxml.compat.xsdata import (
    JsonSerializer as CompatJsonSerializer,
)
from polyxml.compat.xsdata import (
    XmlParser as CompatXmlParser,
)
from polyxml.compat.xsdata import (
    XmlSerializer as CompatXmlSerializer,
)


class Status(Enum):
    PENDING = "pending"
    ACTIVE = "active"
    ARCHIVED = "archived"


@dataclass
class Item:
    sku: str = field(metadata={"name": "skuCode", "type": "Attribute"})
    price: float = field(metadata={"name": "unitPrice", "type": "Element"})


@dataclass
class Customer:
    customer_id: int = field(metadata={"name": "customerId", "type": "Attribute"})
    company_name: str = field(metadata={"name": "companyName", "type": "Element"})
    balance: Decimal = field(metadata={"name": "accountBalance", "type": "Element"})
    is_active: bool = field(metadata={"name": "isActive", "type": "Element"})
    status: Status = field(metadata={"name": "accountStatus", "type": "Element"})
    items: list[Item] = field(default_factory=list, metadata={"name": "Item", "type": "Element"})
    notes: str | None = field(default=None, metadata={"name": "notes", "type": "Element"})


class PydanticItem(BaseModel):
    sku: str = Field(json_schema_extra={"name": "skuCode", "type": "Attribute"})
    price: float = Field(json_schema_extra={"name": "unitPrice", "type": "Element"})


class PydanticCustomer(BaseModel):
    customer_id: int = Field(json_schema_extra={"name": "customerId", "type": "Attribute"})
    company_name: str = Field(json_schema_extra={"name": "companyName", "type": "Element"})
    items: list[PydanticItem] = Field(
        default_factory=list, json_schema_extra={"name": "Item", "type": "Element"}
    )


def test_json_roundtrip_dataclass():
    c = Customer(
        customer_id=101,
        company_name="Acme Inc",
        balance=Decimal("9999.95"),
        is_active=True,
        status=Status.ACTIVE,
        items=[Item(sku="SKU-A", price=19.99), Item(sku="SKU-B", price=49.95)],
        notes=None,
    )

    # 1. Serialize to bytes by alias
    b = polyxml.serialize_json(c, by_alias=True)
    assert b"customerId" in b
    assert b"companyName" in b
    assert b"accountBalance" in b
    assert b"isActive" in b
    assert b"accountStatus" in b

    # 2. Deserialize back
    parsed = polyxml.deserialize_json(b, Customer)
    assert parsed.customer_id == 101
    assert parsed.company_name == "Acme Inc"
    assert parsed.balance == Decimal("9999.95")
    assert parsed.is_active is True
    assert parsed.status == Status.ACTIVE
    assert len(parsed.items) == 2
    assert parsed.items[0].sku == "SKU-A"
    assert parsed.items[0].price == 19.99
    assert parsed.notes is None


def test_json_by_alias_false():
    c = Customer(
        customer_id=202,
        company_name="Initech",
        balance=Decimal("50.00"),
        is_active=False,
        status=Status.PENDING,
    )
    b = polyxml.serialize_json(c, by_alias=False)
    assert b"customer_id" in b
    assert b"company_name" in b
    assert b"balance" in b

    parsed = polyxml.deserialize_json(b, Customer)
    assert parsed.customer_id == 202
    assert parsed.company_name == "Initech"


def test_json_dumps_and_loads():
    c = Customer(
        customer_id=303,
        company_name="Umbrella Corp",
        balance=Decimal("123.45"),
        is_active=True,
        status=Status.ARCHIVED,
    )
    s = polyxml.dumps_json(c, indent=2)
    assert isinstance(s, str)
    assert '  "customerId": 303' in s

    parsed = polyxml.loads_json(s, Customer)
    assert parsed.customer_id == 303
    assert parsed.status == Status.ARCHIVED


def test_json_dual_key_resilience():
    # Test JSON with snake_case property names
    snake_json = '{"customer_id": 404, "company_name": "Globex", "balance": "100.00", "is_active": true, "status": "active"}'
    parsed = polyxml.deserialize_json(snake_json, Customer)
    assert parsed.customer_id == 404
    assert parsed.company_name == "Globex"
    assert parsed.status == Status.ACTIVE


def test_pydantic_json():
    item = PydanticItem(sku="P-1", price=12.5)
    cust = PydanticCustomer(customer_id=505, company_name="Cyberdyne", items=[item])

    json_bytes = polyxml.serialize_json(cust)
    assert b"customerId" in json_bytes
    assert b"skuCode" in json_bytes

    parsed = polyxml.deserialize_json(json_bytes, PydanticCustomer)
    assert parsed.customer_id == 505
    assert parsed.company_name == "Cyberdyne"
    assert len(parsed.items) == 1
    assert parsed.items[0].sku == "P-1"


def test_drop_in_json_serializer_and_parser(tmp_path: pathlib.Path):
    c = Customer(
        customer_id=606,
        company_name="Wayne Enterprises",
        balance=Decimal("1000000.00"),
        is_active=True,
        status=Status.ACTIVE,
    )

    # Test serializer
    serializer = polyxml.JsonSerializer(indent=2)
    rendered = serializer.render(c)
    assert "Wayne Enterprises" in rendered

    # Test parser methods
    parser = polyxml.JsonParser()
    p1 = parser.from_string(rendered, Customer)
    assert p1.customer_id == 606

    p2 = parser.from_bytes(rendered.encode("utf-8"), Customer)
    assert p2.company_name == "Wayne Enterprises"

    # Test from file Path
    file_path = tmp_path / "customer.json"
    file_path.write_text(rendered, encoding="utf-8")
    p3 = parser.parse(file_path, Customer)
    assert p3.customer_id == 606

    # Test from string file path
    p4 = parser.parse(str(file_path), Customer)
    assert p4.customer_id == 606

    # Test from text IO stream
    stream = io.StringIO(rendered)
    p5 = parser.parse(stream, Customer)
    assert p5.customer_id == 606

    # Test from binary IO stream
    b_stream = io.BytesIO(rendered.encode("utf-8"))
    p6 = parser.parse(b_stream, Customer)
    assert p6.customer_id == 606


def test_drop_in_xml_serializer_and_parser(tmp_path: pathlib.Path):
    c = Customer(
        customer_id=707,
        company_name="Stark Industries",
        balance=Decimal("5000.00"),
        is_active=True,
        status=Status.ACTIVE,
    )

    xml_ser = polyxml.XmlSerializer(indent=2)
    xml_str = xml_ser.render(c)
    assert "<Customer" in xml_str
    assert "Stark Industries" in xml_str

    xml_par = polyxml.XmlParser()
    p1 = xml_par.from_string(xml_str, Customer)
    assert p1.customer_id == 707

    p2 = xml_par.from_bytes(xml_str.encode("utf-8"), Customer)
    assert p2.company_name == "Stark Industries"

    # Test from file Path
    file_path = tmp_path / "customer.xml"
    file_path.write_text(xml_str, encoding="utf-8")
    p3 = xml_par.parse(file_path, Customer)
    assert p3.customer_id == 707


def test_compat_xsdata_module():
    js = CompatJsonSerializer()
    jp = CompatJsonParser()
    xs = CompatXmlSerializer()
    xp = CompatXmlParser()

    c = Customer(
        customer_id=808,
        company_name="Oscorp",
        balance=Decimal("250.00"),
        is_active=False,
        status=Status.PENDING,
    )
    json_rendered = js.render(c)
    parsed_json = jp.from_string(json_rendered, Customer)
    assert parsed_json.customer_id == 808

    xml_rendered = xs.render(c)
    parsed_xml = xp.from_string(xml_rendered, Customer)
    assert parsed_xml.customer_id == 808


def test_generated_model_json_codecs():
    @dataclass
    class GeneratedWidget:
        name: str = field(metadata={"name": "widgetName", "type": "Element"})
        cost: float = field(metadata={"name": "widgetCost", "type": "Element"})

        @classmethod
        def from_json(cls, data: bytes | str) -> "GeneratedWidget":
            return polyxml.deserialize_json(data, cls)

        def to_json(self, *, indent: int | None = None, by_alias: bool = True) -> bytes:
            return polyxml.serialize_json(self, indent=indent, by_alias=by_alias)

    w = GeneratedWidget(name="Gizmo", cost=12.99)
    raw = w.to_json(indent=2)
    assert b"widgetName" in raw
    assert b"Gizmo" in raw

    loaded = GeneratedWidget.from_json(raw)
    assert loaded.name == "Gizmo"
    assert loaded.cost == 12.99


def test_to_bytes_error_handling():
    with pytest.raises(TypeError, match="Unsupported XML source type: int"):
        polyxml.deserialize_json(12345, Customer)  # type: ignore[arg-type]


def test_invalid_json_raises():
    with pytest.raises(ValueError):
        polyxml.deserialize_json(b"{not valid json}", Customer)


def test_transcoder_dynamic_bidirectional(tmp_path: pathlib.Path):
    xml_str = '<service id="99" enabled="true"><name>API Gateway</name><port>8080</port><port>8443</port></service>'

    # 1. XML to JSON (dynamic)
    json_bytes = polyxml.xml_to_json(xml_str, indent=2)
    assert b'"@id": 99' in json_bytes
    assert b'"@enabled": true' in json_bytes
    assert b'"name": "API Gateway"' in json_bytes
    assert b"8080" in json_bytes
    assert b"8443" in json_bytes

    # 2. JSON to XML (dynamic)
    roundtrip_xml = polyxml.json_to_xml(json_bytes, indent=2)
    assert b"<service" in roundtrip_xml
    assert b'id="99"' in roundtrip_xml
    assert b'enabled="true"' in roundtrip_xml
    assert b"<name>API Gateway</name>" in roundtrip_xml
    assert b"<port>8080</port>" in roundtrip_xml

    # Test with pathlib.Path and IO stream
    xml_file = tmp_path / "service.xml"
    xml_file.write_bytes(xml_str.encode("utf-8"))
    json_from_file = polyxml.xml_to_json(xml_file)
    assert b'"@id":99' in json_from_file

    stream = io.BytesIO(json_from_file)
    xml_from_stream = polyxml.json_to_xml(stream)
    assert b'id="99"' in xml_from_stream


def test_transcoder_model_guided():
    xml_data = (
        b'<Customer customerId="456">'
        b"<companyName>Cyberdyne Systems</companyName>"
        b"<accountBalance>1250000.50</accountBalance>"
        b"<isActive>true</isActive>"
        b"<accountStatus>active</accountStatus>"
        b'<Item skuCode="T-800"><unitPrice>99999.99</unitPrice></Item>'
        b"</Customer>"
    )

    # 1. XML -> JSON guided by Model (dataclass)
    json_bytes = polyxml.xml_to_json(xml_data, Customer, indent=2)
    assert b'"customerId": 456' in json_bytes
    assert b'"companyName": "Cyberdyne Systems"' in json_bytes
    assert b'"accountBalance": "1250000.50"' in json_bytes
    assert b'"skuCode": "T-800"' in json_bytes

    # Using model= kwarg alias
    json_bytes_kwarg = polyxml.xml_to_json(xml_data, model=Customer, by_alias=True)
    assert b'"customerId":456' in json_bytes_kwarg

    # 2. JSON -> XML guided by Model
    xml_roundtrip = polyxml.json_to_xml(json_bytes, model=Customer, root="Customer", indent=2)
    assert b'<Customer customerId="456">' in xml_roundtrip
    assert b"<companyName>Cyberdyne Systems</companyName>" in xml_roundtrip
    assert b'<skuCode="T-800"' in xml_roundtrip or b'skuCode="T-800"' in xml_roundtrip


def test_transcoder_schema_path_guided(tmp_path: pathlib.Path):
    xsd_content = """<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="Product" type="ProductType"/>
    <xs:complexType name="ProductType">
        <xs:sequence>
            <xs:element name="title" type="xs:string"/>
            <xs:element name="price" type="xs:decimal"/>
        </xs:sequence>
        <xs:attribute name="code" type="xs:string" use="required"/>
    </xs:complexType>
</xs:schema>"""

    xsd_file = tmp_path / "product.xsd"
    xsd_file.write_text(xsd_content, encoding="utf-8")

    xml_content = (
        '<Product code="PROD-77"><title>Mechanical Keyboard</title><price>129.99</price></Product>'
    )

    # 1. XML -> JSON with schema_path
    json_bytes = polyxml.xml_to_json(xml_content, schema_path=xsd_file, root="Product", indent=2)
    assert b'"code": "PROD-77"' in json_bytes
    assert b'"title": "Mechanical Keyboard"' in json_bytes
    assert b'"price": "129.99"' in json_bytes

    # 2. JSON -> XML with schema_path and str path
    xml_bytes = polyxml.json_to_xml(json_bytes, schema_path=str(xsd_file), root="Product", indent=2)
    assert b'code="PROD-77"' in xml_bytes
    assert b"<title>Mechanical Keyboard</title>" in xml_bytes
    assert b"<price>129.99</price>" in xml_bytes


def test_transcoder_error_handling(tmp_path: pathlib.Path):
    # Non-existent schema file
    with pytest.raises(ValueError, match="Failed to read schema file"):
        polyxml.xml_to_json(b"<foo/>", schema_path="non_existent_schema.xsd")

    # Invalid schema XML
    bad_xsd = tmp_path / "bad.xsd"
    bad_xsd.write_text("not xml", encoding="utf-8")
    with pytest.raises(ValueError, match="Failed to (parse schema|build schema)"):
        polyxml.xml_to_json(b"<foo/>", schema_path=bad_xsd)

    # Empty XML
    with pytest.raises(ValueError):
        polyxml.xml_to_json(b"", indent=2)

    # Invalid JSON
    with pytest.raises(ValueError):
        polyxml.json_to_xml(b"not json", indent=2)
