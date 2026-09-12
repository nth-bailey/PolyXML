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

import pathlib
from collections.abc import Iterator
from decimal import Decimal
from typing import IO

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


def _enc_hook(obj: object) -> str:
    if isinstance(obj, pathlib.Path | Decimal):
        return str(obj)
    if hasattr(obj, "from_string") and hasattr(obj, "val"):
        return str(obj)
    raise NotImplementedError(
        f"PolyXML binary serializer cannot serialize object of type {type(obj).__name__}"
    )


def _dec_hook(target_type: type[object], obj: object) -> object:
    if target_type is pathlib.Path:
        return pathlib.Path(str(obj))
    if hasattr(target_type, "from_string"):
        return target_type.from_string(str(obj))
    raise NotImplementedError(
        f"PolyXML binary deserializer cannot deserialize object into type {target_type.__name__}"
    )


def dumps_binary(obj: object) -> bytes:
    """Serialize a model or dataclass into a high-throughput binary MessagePack buffer.

    Args:
        obj: Python dataclass, Pydantic model, or object.

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
    return msgspec.msgpack.encode(obj, enc_hook=_enc_hook)


def loads_binary[T](data: bytes, target_type: type[T] | None = None) -> T | object:
    """Deserialize binary MessagePack bytes into a strongly-typed model or object.

    Args:
        data: Binary MessagePack bytes to decode.
        target_type: Optional target dataclass or model class. If omitted, returns dynamic structure.

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
        return msgspec.msgpack.decode(data, type=target_type, dec_hook=_dec_hook)
    return msgspec.msgpack.decode(data, dec_hook=_dec_hook)


__all__ = [
    "__version__",
    "deserialize",
    "dumps_binary",
    "iterparse",
    "loads_binary",
    "serialize",
]
