"""PolyXML: High-performance, polyglot XML data-binding engine."""

from collections.abc import Iterator

from polyxml._polyxml import (
    deserialize as _deserialize,
)  # type: ignore[import-not-found]
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


def deserialize[T](source: bytes | str, target_type: type[T]) -> T:
    """Deserialize XML bytes or string into a Python dataclass or model instance.

    Args:
        source: XML content as raw bytes or a string.
        target_type: The target dataclass or model class.

    Returns:
        The deserialized model instance.
    """
    if isinstance(source, str):
        source = source.encode("utf-8")
    return _deserialize(source, target_type)


def iterparse[T](
    source: bytes | str,
    target_type: type[T],
    tag: str | None = None,
) -> Iterator[T]:
    """Stream and deserialize XML elements one-by-one with O(1) constant memory.

    Args:
        source: XML content as raw bytes or a string.
        target_type: The target dataclass or model class for each element.
        tag: Optional XML element tag name to match. Defaults to the model schema name.

    Yields:
        Deserialized model instances as they are streamed.
    """
    if isinstance(source, str):
        source = source.encode("utf-8")
    return _iterparse(source, target_type, tag)


def serialize(obj: object, *, indent: int | None = None) -> bytes:
    """Serialize a strongly-typed model instance back into XML bytes.

    Args:
        obj: Python dataclass or Pydantic model instance.
        indent: Optional indentation size in spaces for pretty-printing.

    Returns:
        UTF-8 encoded XML bytes representing the model instance.
    """
    return _serialize(obj, indent=indent)


__all__ = ["__version__", "deserialize", "iterparse", "serialize"]
