"""Deterministic XML payload generators for PolyXML benchmarking."""

from __future__ import annotations


def generate_sensor_xml(
    sensor_id: int = 1001,
    temp: float = 23.75,
    status: str = "OPERATIONAL",
    active: bool = True,
) -> bytes:
    """Generate a micro telemetry XML payload (~100 bytes)."""
    return (
        f'<Sensor id="{sensor_id}">'
        f"<temp>{temp:.2f}</temp>"
        f"<status>{status}</status>"
        f"<active>{'true' if active else 'false'}</active>"
        f"</Sensor>"
    ).encode()


def generate_order_xml(num_items: int = 10, order_id: str = "ORD-2026-9901") -> bytes:
    """Generate a medium nested order XML payload (~1-3 KB)."""
    items_xml = "".join(
        f'<item id="{i + 1}" qty="{(i % 5) + 1}">'
        f"<name>Part-{i + 1}</name>"
        f"<price>{((i + 1) * 12.5):.2f}</price>"
        f"<available>true</available>"
        f"</item>"
        for i in range(num_items)
    )
    return (
        f'<Order id="{order_id}"><customer>Acme Aerospace</customer>{items_xml}</Order>'
    ).encode()


def generate_catalog_xml(num_items: int = 1000) -> bytes:
    """Generate a high-volume streaming XML payload (65 KB to 6.5 MB)."""
    items_xml = "".join(
        f'<item id="{i}"><name>CatalogItem-{i}</name><price>{(i * 1.75):.2f}</price></item>'
        for i in range(num_items)
    )
    return f"<Catalog>{items_xml}</Catalog>".encode()
