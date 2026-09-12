"""PolyXML: High-performance, polyglot native XML data-binding engine.

QUICKSTART FOR AI AGENTS & DEVELOPERS:

1. Deserialize XML (string, bytes, or file Path) into a typed Dataclass or Pydantic model:
    >>> import polyxml
    >>> from dataclasses import dataclass, field
    >>>
    >>> @dataclass
    >>> class User:
    ...     name: str = field(metadata={"type": "Element"})
    >>>
    >>> user = polyxml.deserialize("<User><name>Alice</name></User>", User)
    >>> user.name
    'Alice'

2. Stream large XML files with O(1) constant memory (Streaming / Facet Extraction):
    >>> for user in polyxml.iterparse("massive_users.xml", User, tag="User"):
    ...     process(user)

3. Serialize a model instance back into XML bytes:
    >>> xml_bytes = polyxml.serialize(user, indent=2)

See `AGENT_GUIDE.md` or https://nth-bailey.github.io/PolyXML/ for complete documentation.
"""

import importlib
import pathlib
from collections import UserString
from collections.abc import Iterator
from dataclasses import is_dataclass
from decimal import Decimal
from enum import Enum
from typing import IO
from xml.etree.ElementTree import QName

from polyxml._polyxml import (  # type: ignore[import-not-found]
    deserialize as _deserialize,
)
from polyxml._polyxml import (
    iterparse as _iterparse,
)
from polyxml._polyxml import (
    serialize as _serialize,
)
from polyxml._polyxml import (
    version as _version,
)

__version__: str = _version()


def _to_bytes(source: bytes | str | pathlib.Path | IO[bytes] | IO[str]) -> bytes:
    if isinstance(source, bytes):
        return source
    if isinstance(source, str):
        if source.lstrip().startswith("<"):
            return source.encode("utf-8")
        path = pathlib.Path(source)
        if path.is_file():
            return path.read_bytes()
        return source.encode("utf-8")
    if isinstance(source, pathlib.Path):
        return source.read_bytes()
    if hasattr(source, "read"):
        data = source.read()
        return data.encode("utf-8") if isinstance(data, str) else data
    raise TypeError(
        f"Unsupported XML source type: {type(source).__name__}. "
        "Expected bytes, str (XML string or file path), pathlib.Path, or a binary/text IO stream."
    )


def deserialize[T](
    source: bytes | str | pathlib.Path | IO[bytes] | IO[str], target_type: type[T]
) -> T:
    """Deserialize XML bytes, string, file path, or stream into a typed model.

    Args:
        source: XML content as raw bytes, string, Path, or file stream.
        target_type: The target dataclass or model class.

    Returns:
        The deserialized model instance.
    """
    return _deserialize(_to_bytes(source), target_type)


def iterparse[T](
    source: bytes | str | pathlib.Path | IO[bytes] | IO[str],
    target_type: type[T],
    tag: str | None = None,
) -> Iterator[T]:
    """Stream and deserialize XML elements one-by-one with O(1) constant memory.

    Args:
        source: XML content as raw bytes, string, Path, or file stream.
        target_type: The target dataclass or model class for each element.
        tag: Optional XML element tag name to match. Defaults to the model schema name.

    Yields:
        Deserialized model instances as they are streamed.
    """
    return _iterparse(_to_bytes(source), target_type, tag)


def serialize(obj: object, *, indent: int | None = None) -> bytes:
    """Serialize a strongly-typed model instance back into XML bytes.

    Args:
        obj: Python dataclass or Pydantic model instance.
        indent: Optional indentation size in spaces for pretty-printing.

    Returns:
        UTF-8 encoded XML bytes representing the model instance.
    """
    return _serialize(obj, indent=indent)


try:
    import msgspec

    _HAS_MSGSPEC = True
except ImportError:  # pragma: no cover
    msgspec = None  # type: ignore[assignment]
    _HAS_MSGSPEC = False

_CLASS_CACHE: dict[str, type[object]] = {}


def _resolve_class(class_identifier: str) -> type[object] | None:
    if not class_identifier:
        return None
    if class_identifier in _CLASS_CACHE:
        return _CLASS_CACHE[class_identifier]
    if ":" in class_identifier:
        module_name, class_name = class_identifier.rsplit(":", 1)
        try:
            mod = importlib.import_module(module_name)
            resolved = getattr(mod, class_name)
            _CLASS_CACHE[class_identifier] = resolved
            return resolved
        except (ImportError, AttributeError):
            return None
    return None


def _enc_hook(obj: object) -> object:
    if isinstance(obj, pathlib.Path | Decimal):
        return str(obj)
    if isinstance(obj, Enum):
        return obj.value
    if isinstance(obj, QName):
        return str(obj)
    if isinstance(obj, UserString):
        return str(obj)
    if hasattr(obj, "model_dump") and callable(obj.model_dump):
        return obj.model_dump()
    if hasattr(obj, "from_string"):
        return str(obj)
    raise NotImplementedError(
        f"PolyXML binary serializer cannot serialize object of type {type(obj).__name__}"
    )


def _dec_hook(target_type: type[object], obj: object) -> object:
    if target_type is pathlib.Path:
        return pathlib.Path(str(obj))
    if target_type is Decimal:
        return Decimal(str(obj))
    if isinstance(target_type, type) and issubclass(target_type, Enum):
        return target_type(obj)
    if target_type is QName or (isinstance(target_type, type) and issubclass(target_type, QName)):
        return QName(str(obj))
    if hasattr(target_type, "model_validate") and callable(target_type.model_validate):
        return target_type.model_validate(obj)
    if hasattr(target_type, "from_string"):
        return target_type.from_string(str(obj))
    if isinstance(target_type, type) and issubclass(target_type, UserString):
        return target_type(str(obj))
    raise NotImplementedError(
        f"PolyXML binary deserializer cannot deserialize object into type {target_type.__name__}"
    )


def dumps_binary(obj: object, *, tag_class: bool = False) -> bytes:
    """Serialize a model or dataclass into a high-throughput binary MessagePack buffer.

    Args:
        obj: Python dataclass, Pydantic model, or object.
        tag_class: When True, embeds the class identifier in the binary payload
            allowing loads_binary to reconstruct the exact model class dynamically.

    Returns:
        Compact binary MessagePack bytes.

    Raises:
        RuntimeError: If msgspec is not installed.
    """
    if not _HAS_MSGSPEC:  # pragma: no cover
        raise RuntimeError(
            "PolyXML binary serialization requires 'msgspec'. "
            "Install it via: pip install 'polyxml[msgpack]' or pip install msgspec"
        )
    if tag_class:
        if is_dataclass(obj) or hasattr(obj.__class__, "model_fields"):
            cls = obj.__class__
            identifier = f"{cls.__module__}:{cls.__qualname__}"
            _CLASS_CACHE[identifier] = cls
            payload = msgspec.msgpack.encode(obj, enc_hook=_enc_hook)
            return msgspec.msgpack.encode((identifier, payload))
        payload = msgspec.msgpack.encode(obj, enc_hook=_enc_hook)
        return msgspec.msgpack.encode(("", payload))

    return msgspec.msgpack.encode(obj, enc_hook=_enc_hook)


def loads_binary[T](data: bytes, target_type: type[T] | None = None) -> T | object:
    """Deserialize binary MessagePack bytes into a strongly-typed model or object.

    Args:
        data: Binary MessagePack bytes to decode.
        target_type: Optional target dataclass or model class. If omitted and the
            payload was serialized with tag_class=True, dynamically reconstructs the
            original model instance. Otherwise returns a dynamic dictionary/structure.

    Returns:
        The deserialized model instance or object.

    Raises:
        RuntimeError: If msgspec is not installed.
    """
    if not _HAS_MSGSPEC:  # pragma: no cover
        raise RuntimeError(
            "PolyXML binary deserialization requires 'msgspec'. "
            "Install it via: pip install 'polyxml[msgpack]' or pip install msgspec"
        )
    if target_type is not None:
        try:
            return msgspec.msgpack.decode(data, type=target_type, dec_hook=_dec_hook)
        except (msgspec.DecodeError, msgspec.ValidationError):
            try:
                decoded = msgspec.msgpack.decode(data)
                if (
                    isinstance(decoded, (list, tuple))
                    and len(decoded) == 2
                    and isinstance(decoded[0], str)
                    and isinstance(decoded[1], (bytes, bytearray))
                ):
                    return msgspec.msgpack.decode(decoded[1], type=target_type, dec_hook=_dec_hook)
            except Exception:
                pass
            raise

    try:
        decoded = msgspec.msgpack.decode(data)
        if (
            isinstance(decoded, (list, tuple))
            and len(decoded) == 2
            and isinstance(decoded[0], str)
            and isinstance(decoded[1], (bytes, bytearray))
        ):
            tag, payload = decoded
            if tag:
                resolved_cls = _resolve_class(tag)
                if resolved_cls is not None:
                    return msgspec.msgpack.decode(payload, type=resolved_cls, dec_hook=_dec_hook)
            return msgspec.msgpack.decode(payload, dec_hook=_dec_hook)
        return decoded
    except Exception:
        return msgspec.msgpack.decode(data, dec_hook=_dec_hook)


__all__ = [
    "__version__",
    "deserialize",
    "dumps_binary",
    "iterparse",
    "loads_binary",
    "serialize",
]
