"""Drop-in xsdata compatibility layer for PolyXML.

Allows seamless migration from xsdata with zero code changes:
    # Before:
    # from xsdata.formats.dataclass.serializers import JsonSerializer, XmlSerializer
    # from xsdata.formats.dataclass.parsers import JsonParser, XmlParser

    # After:
    from polyxml.compat.xsdata import JsonSerializer, JsonParser, XmlSerializer, XmlParser
"""

from polyxml import (
    JsonParser,
    JsonSerializer,
    XmlParser,
    XmlSerializer,
    deserialize,
    deserialize_json,
    dumps_json,
    loads_json,
    serialize,
    serialize_json,
)

__all__ = [
    "JsonParser",
    "JsonSerializer",
    "XmlParser",
    "XmlSerializer",
    "deserialize",
    "deserialize_json",
    "dumps_json",
    "loads_json",
    "serialize",
    "serialize_json",
]
