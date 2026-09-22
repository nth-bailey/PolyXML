"""Data models for PolyXML, xsdata, and pydantic-xml comparative benchmarks."""

from dataclasses import dataclass, field

from pydantic import BaseModel, Field

# -----------------------------------------------------------------------------
# 1. PolyXML Models (Dataclass & Pydantic v2)
# -----------------------------------------------------------------------------


@dataclass
class PolySensor:
    id: int = field(metadata={"type": "Attribute"})
    temp: float = field(metadata={"type": "Element"})
    status: str = field(metadata={"type": "Element"})
    active: bool = field(metadata={"type": "Element"})


@dataclass
class PolyCatalogItem:
    id: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})
    price: float = field(metadata={"type": "Element"})


@dataclass
class PolyCatalog:
    items: list[PolyCatalogItem] = field(metadata={"type": "Element", "name": "item"})


@dataclass
class PolyOrderItem:
    id: int = field(metadata={"type": "Attribute"})
    qty: int = field(metadata={"type": "Attribute"})
    name: str = field(metadata={"type": "Element"})
    price: float = field(metadata={"type": "Element"})
    available: bool = field(metadata={"type": "Element"})


@dataclass
class PolyOrder:
    id: str = field(metadata={"type": "Attribute"})
    customer: str = field(metadata={"type": "Element"})
    items: list[PolyOrderItem] = field(metadata={"type": "Element", "name": "item"})


class PydanticSensor(BaseModel):
    id: int = Field(..., json_schema_extra={"type": "Attribute"})
    temp: float = Field(..., json_schema_extra={"type": "Element"})
    status: str = Field(..., json_schema_extra={"type": "Element"})
    active: bool = Field(..., json_schema_extra={"type": "Element"})


class PydanticCatalogItem(BaseModel):
    id: int = Field(..., json_schema_extra={"type": "Attribute"})
    name: str = Field(..., json_schema_extra={"type": "Element"})
    price: float = Field(..., json_schema_extra={"type": "Element"})


class PydanticCatalog(BaseModel):
    items: list[PydanticCatalogItem] = Field(
        ..., json_schema_extra={"type": "Element", "name": "item"}
    )


# -----------------------------------------------------------------------------
# 2. xsdata Models
# -----------------------------------------------------------------------------

try:
    from xsdata.formats.dataclass.parsers import XmlParser as XsXmlParser
    from xsdata.formats.dataclass.serializers import XmlSerializer as XsXmlSerializer

    HAS_XSDATA = True

    @dataclass
    class XsSensor:
        id: int = field(metadata={"type": "Attribute"})
        temp: float = field(metadata={"type": "Element"})
        status: str = field(metadata={"type": "Element"})
        active: bool = field(metadata={"type": "Element"})

    @dataclass
    class XsCatalogItem:
        id: int = field(metadata={"type": "Attribute"})
        name: str = field(metadata={"type": "Element"})
        price: float = field(metadata={"type": "Element"})

    @dataclass
    class XsCatalog:
        item: list[XsCatalogItem] = field(
            default_factory=list, metadata={"type": "Element"}
        )

    @dataclass
    class XsOrderItem:
        id: int = field(metadata={"type": "Attribute"})
        qty: int = field(metadata={"type": "Attribute"})
        name: str = field(metadata={"type": "Element"})
        price: float = field(metadata={"type": "Element"})
        available: bool = field(metadata={"type": "Element"})

    @dataclass
    class XsOrder:
        id: str = field(metadata={"type": "Attribute"})
        customer: str = field(metadata={"type": "Element"})
        items: list[XsOrderItem] = field(
            default_factory=list, metadata={"type": "Element", "name": "item"}
        )

except ImportError:
    HAS_XSDATA = False
    XsXmlParser = None  # type: ignore
    XsXmlSerializer = None  # type: ignore
    XsSensor = None  # type: ignore
    XsCatalogItem = None  # type: ignore
    XsCatalog = None  # type: ignore
    XsOrderItem = None  # type: ignore
    XsOrder = None  # type: ignore

# -----------------------------------------------------------------------------
# 3. pydantic-xml Models
# -----------------------------------------------------------------------------

try:
    from pydantic_xml import BaseXmlModel, attr, element

    HAS_PXML = True

    class PxmlSensor(BaseXmlModel, tag="Sensor"):
        id: int = attr()
        temp: float = element()
        status: str = element()
        active: bool = element()

    class PxmlCatalogItem(BaseXmlModel, tag="item"):
        id: int = attr()
        name: str = element()
        price: float = element()

    class PxmlCatalog(BaseXmlModel, tag="Catalog"):
        items: list[PxmlCatalogItem] = element(tag="item", default_factory=list)

except ImportError:
    HAS_PXML = False
    PxmlSensor = None  # type: ignore
    PxmlCatalogItem = None  # type: ignore
    PxmlCatalog = None  # type: ignore

# -----------------------------------------------------------------------------
# 4. declxml Processors
# -----------------------------------------------------------------------------

try:
    import declxml as xml_decl

    HAS_DECLXML = True

    DECLXML_SENSOR_PROC = xml_decl.dictionary(
        "Sensor",
        [
            xml_decl.integer(".", attribute="id"),
            xml_decl.floating_point("temp"),
            xml_decl.string("status"),
            xml_decl.boolean("active"),
        ],
    )

    DECLXML_CATALOG_ITEM_PROC = xml_decl.dictionary(
        "item",
        [
            xml_decl.integer(".", attribute="id"),
            xml_decl.string("name"),
            xml_decl.floating_point("price"),
        ],
    )

    DECLXML_CATALOG_PROC = xml_decl.dictionary(
        "Catalog",
        [xml_decl.array(DECLXML_CATALOG_ITEM_PROC, alias="item")],
    )

    DECLXML_ORDER_ITEM_PROC = xml_decl.dictionary(
        "item",
        [
            xml_decl.integer(".", attribute="id"),
            xml_decl.integer(".", attribute="qty"),
            xml_decl.string("name"),
            xml_decl.floating_point("price"),
            xml_decl.boolean("available"),
        ],
    )

    DECLXML_ORDER_PROC = xml_decl.dictionary(
        "Order",
        [
            xml_decl.string(".", attribute="id"),
            xml_decl.string("customer"),
            xml_decl.array(DECLXML_ORDER_ITEM_PROC, alias="items"),
        ],
    )

except ImportError:
    HAS_DECLXML = False
    DECLXML_SENSOR_PROC = None  # type: ignore
    DECLXML_CATALOG_PROC = None  # type: ignore
    DECLXML_ORDER_PROC = None  # type: ignore
