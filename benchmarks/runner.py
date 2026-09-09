"""Benchmark execution engine for PolyXML and comparative parsers."""

from __future__ import annotations

import gc
import json
import statistics
import time
import tracemalloc
import xml.etree.ElementTree as ET
from collections.abc import Callable
from dataclasses import asdict, dataclass
from typing import Any

import polyxml
from rich.console import Console
from rich.table import Table

from benchmarks.generators import (
    generate_catalog_xml,
    generate_order_xml,
    generate_sensor_xml,
)
from benchmarks.models import (
    HAS_PXML,
    HAS_XSDATA,
    PolyCatalog,
    PolyOrder,
    PolySensor,
    PydanticCatalog,
    PydanticSensor,
)

try:
    from lxml import etree as lxml_ET
    from lxml import objectify as lxml_objectify

    HAS_LXML = True
except ImportError:
    HAS_LXML = False
    lxml_ET = None
    lxml_objectify = None

try:
    import xmltodict

    HAS_XMLTODICT = True
except ImportError:
    HAS_XMLTODICT = False
    xmltodict = None

try:
    import defusedxml.ElementTree as defused_ET

    HAS_DEFUSEDXML = True
except ImportError:
    HAS_DEFUSEDXML = False
    defused_ET = None

try:
    import untangle

    HAS_UNTANGLE = True
except ImportError:
    HAS_UNTANGLE = False
    untangle = None

try:
    import declxml as xml_decl

    from benchmarks.models import (
        DECLXML_CATALOG_PROC,
        DECLXML_ORDER_PROC,
        DECLXML_SENSOR_PROC,
        HAS_DECLXML,
    )
except ImportError:
    HAS_DECLXML = False
    xml_decl = None
    DECLXML_SENSOR_PROC = None
    DECLXML_CATALOG_PROC = None
    DECLXML_ORDER_PROC = None

if HAS_XSDATA:
    from xsdata.formats.dataclass.parsers import XmlParser as XsXmlParser
    from xsdata.formats.dataclass.serializers import XmlSerializer as XsXmlSerializer

    from benchmarks.models import (
        XsCatalog,
        XsCatalogItem,
        XsOrder,
        XsOrderItem,
        XsSensor,
    )

if HAS_PXML:
    from benchmarks.models import PxmlCatalog, PxmlSensor


@dataclass
class BenchmarkMetric:
    workload: str
    engine: str
    target_type: str  # e.g., "Typed Dataclass", "Typed Pydantic", "Untyped DOM"
    operation: str  # "Deserialize" or "Serialize"
    payload_size_bytes: int
    iterations: int
    min_ms: float
    median_ms: float
    mean_ms: float
    p95_ms: float
    throughput_mbs: float
    peak_mem_kib: float
    speedup_vs_baseline: float | None = None


class BenchmarkRunner:
    def __init__(self, console: Console | None = None):
        self.console = console or Console(width=140)
        self.results: list[BenchmarkMetric] = []

    def measure(
        self,
        fn: Callable[[], Any],
        payload_bytes: int,
        iterations: int = 50,
        warmup: int = 5,
    ) -> tuple[float, float, float, float, float, float]:
        """Execute warmup and timed iterations, returning statistics and peak memory."""
        # 1. Warmup
        for _ in range(warmup):
            fn()

        # 2. Measure Peak Memory
        gc.collect()
        tracemalloc.start()
        fn()
        _current_mem, peak_mem = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        gc.collect()
        peak_kib = peak_mem / 1024.0

        # 3. Timed Iterations
        times_ms: list[float] = []
        for _ in range(iterations):
            gc.disable()
            t0 = time.perf_counter_ns()
            fn()
            t1 = time.perf_counter_ns()
            gc.enable()
            times_ms.append((t1 - t0) / 1_000_000.0)

        min_ms = min(times_ms)
        median_ms = statistics.median(times_ms)
        mean_ms = statistics.mean(times_ms)
        sorted_times = sorted(times_ms)
        p95_idx = int(0.95 * len(sorted_times))
        p95_ms = sorted_times[min(p95_idx, len(sorted_times) - 1)]

        # Throughput in MB/s based on min_ms
        sec = min_ms / 1000.0
        mb = payload_bytes / (1024.0 * 1024.0)
        throughput = mb / sec if sec > 0 else 0.0

        return min_ms, median_ms, mean_ms, p95_ms, throughput, peak_kib

    def run_workload_sensor(self, iterations: int = 200) -> list[BenchmarkMetric]:
        """Benchmark small sensor telemetry payload (~100 bytes)."""
        xml = generate_sensor_xml()
        size = len(xml)
        workload = "Sensor Micro (~100B)"
        metrics: list[BenchmarkMetric] = []

        # Target instances for serialization
        poly_dc_instance = polyxml.deserialize(xml, PolySensor)
        poly_pydantic_instance = polyxml.deserialize(xml, PydanticSensor)

        # --- Deserialization ---
        # 1. PolyXML (Dataclass)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.deserialize(xml, PolySensor), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML",
                "Typed Dataclass",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 2. PolyXML (Pydantic v2)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.deserialize(xml, PydanticSensor), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML (Pydantic)",
                "Typed Pydantic",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 3. xml.etree.ElementTree (DOM)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: ET.fromstring(xml), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "ElementTree",
                "Untyped DOM",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 4. defusedxml (Secure DOM)
        if HAS_DEFUSEDXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: defused_ET.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "defusedxml",
                    "Secure DOM",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 5. lxml (DOM)
        if HAS_LXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_ET.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml",
                    "Untyped DOM",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 6. lxml.objectify (C Dynamic Object)
        if HAS_LXML and lxml_objectify is not None:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_objectify.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml.objectify",
                    "C Dynamic Object",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 7. xmltodict (Untyped Dict)
        if HAS_XMLTODICT:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xmltodict.parse(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xmltodict",
                    "Untyped Dict",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 8. xsdata
        if HAS_XSDATA:
            xs_parser = XsXmlParser()
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xs_parser.from_bytes(xml, XsSensor), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xsdata",
                    "Typed Dataclass",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 9. pydantic-xml
        if HAS_PXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: PxmlSensor.from_xml(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "pydantic-xml",
                    "Typed Pydantic",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 10. declxml (Declarative Dict)
        if HAS_DECLXML and DECLXML_SENSOR_PROC is not None:
            xml_str = xml.decode("utf-8")
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xml_decl.parse_from_string(DECLXML_SENSOR_PROC, xml_str),
                size,
                iterations,
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "declxml",
                    "Declarative Dict",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 11. untangle (Dynamic Object)
        if HAS_UNTANGLE:
            xml_str = xml.decode("utf-8")
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: untangle.parse(xml_str), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "untangle",
                    "Dynamic Object",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # --- Serialization ---
        # 1. PolyXML (Dataclass)
        ser_xml = polyxml.serialize(poly_dc_instance)
        ser_size = len(ser_xml)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.serialize(poly_dc_instance), ser_size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML",
                "Typed Dataclass",
                "Serialize",
                ser_size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 2. PolyXML (Pydantic v2)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.serialize(poly_pydantic_instance), ser_size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML (Pydantic)",
                "Typed Pydantic",
                "Serialize",
                ser_size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 3. lxml.objectify
        if HAS_LXML and lxml_objectify is not None:
            obj_inst = lxml_objectify.fromstring(xml)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_ET.tostring(obj_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml.objectify",
                    "C Dynamic Object",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 4. xmltodict
        if HAS_XMLTODICT:
            d_inst = xmltodict.parse(xml)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xmltodict.unparse(d_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xmltodict",
                    "Untyped Dict",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 5. xsdata
        if HAS_XSDATA:
            xs_serializer = XsXmlSerializer()
            xs_inst = XsSensor(id=1001, temp=23.75, status="OPERATIONAL", active=True)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xs_serializer.render(xs_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xsdata",
                    "Typed Dataclass",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 6. pydantic-xml
        if HAS_PXML:
            pxml_inst = PxmlSensor(
                id=1001, temp=23.75, status="OPERATIONAL", active=True
            )
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: pxml_inst.to_xml(), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "pydantic-xml",
                    "Typed Pydantic",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 7. declxml
        if HAS_DECLXML and DECLXML_SENSOR_PROC is not None:
            xml_str = xml.decode("utf-8")
            dec_inst = xml_decl.parse_from_string(DECLXML_SENSOR_PROC, xml_str)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xml_decl.serialize_to_string(DECLXML_SENSOR_PROC, dec_inst),
                ser_size,
                iterations,
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "declxml",
                    "Declarative Dict",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        return metrics

    def run_workload_order(
        self, num_items: int = 10, iterations: int = 100
    ) -> list[BenchmarkMetric]:
        """Benchmark enterprise nested order payload (~2 KB)."""
        xml = generate_order_xml(num_items=num_items)
        size = len(xml)
        workload = f"Order Nested ({num_items} items, {size}B)"
        metrics: list[BenchmarkMetric] = []

        poly_dc_instance = polyxml.deserialize(xml, PolyOrder)

        # --- Deserialization ---
        # 1. PolyXML (Dataclass)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.deserialize(xml, PolyOrder), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML",
                "Typed Dataclass",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 2. ElementTree (DOM)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: ET.fromstring(xml), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "ElementTree",
                "Untyped DOM",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 3. defusedxml (Secure DOM)
        if HAS_DEFUSEDXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: defused_ET.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "defusedxml",
                    "Secure DOM",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 4. lxml (DOM)
        if HAS_LXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_ET.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml",
                    "Untyped DOM",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 5. lxml.objectify (C Dynamic Object)
        if HAS_LXML and lxml_objectify is not None:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_objectify.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml.objectify",
                    "C Dynamic Object",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 6. xmltodict (Untyped Dict)
        if HAS_XMLTODICT:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xmltodict.parse(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xmltodict",
                    "Untyped Dict",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 7. xsdata
        if HAS_XSDATA:
            xs_parser = XsXmlParser()
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xs_parser.from_bytes(xml, XsOrder), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xsdata",
                    "Typed Dataclass",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 8. declxml (Declarative Dict)
        if HAS_DECLXML and DECLXML_ORDER_PROC is not None:
            xml_str = xml.decode("utf-8")
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xml_decl.parse_from_string(DECLXML_ORDER_PROC, xml_str),
                size,
                iterations,
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "declxml",
                    "Declarative Dict",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 9. untangle (Dynamic Object)
        if HAS_UNTANGLE:
            xml_str = xml.decode("utf-8")
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: untangle.parse(xml_str), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "untangle",
                    "Dynamic Object",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # --- Serialization ---
        # 1. PolyXML (Dataclass)
        ser_xml = polyxml.serialize(poly_dc_instance)
        ser_size = len(ser_xml)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.serialize(poly_dc_instance), ser_size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML",
                "Typed Dataclass",
                "Serialize",
                ser_size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 2. lxml.objectify
        if HAS_LXML and lxml_objectify is not None:
            obj_inst = lxml_objectify.fromstring(xml)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_ET.tostring(obj_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml.objectify",
                    "C Dynamic Object",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 3. xmltodict
        if HAS_XMLTODICT:
            d_inst = xmltodict.parse(xml)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xmltodict.unparse(d_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xmltodict",
                    "Untyped Dict",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 4. xsdata
        if HAS_XSDATA:
            xs_serializer = XsXmlSerializer()
            xs_items = [
                XsOrderItem(
                    id=i + 1,
                    qty=(i % 5) + 1,
                    name=f"Part-{i + 1}",
                    price=(i + 1) * 12.5,
                    available=True,
                )
                for i in range(num_items)
            ]
            xs_order_inst = XsOrder(
                id="ORD-2026-9901", customer="Acme Aerospace", items=xs_items
            )
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xs_serializer.render(xs_order_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xsdata",
                    "Typed Dataclass",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 5. declxml
        if HAS_DECLXML and DECLXML_ORDER_PROC is not None:
            xml_str = xml.decode("utf-8")
            dec_inst = xml_decl.parse_from_string(DECLXML_ORDER_PROC, xml_str)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xml_decl.serialize_to_string(DECLXML_ORDER_PROC, dec_inst),
                ser_size,
                iterations,
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "declxml",
                    "Declarative Dict",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        return metrics

    def run_workload_catalog(
        self, num_items: int = 1000, iterations: int = 30
    ) -> list[BenchmarkMetric]:
        """Benchmark batch streaming catalog payload."""
        xml = generate_catalog_xml(num_items)
        size = len(xml)
        workload = f"Catalog Batch ({num_items:,} items, {size / 1024:.1f} KB)"
        metrics: list[BenchmarkMetric] = []

        poly_dc_instance = polyxml.deserialize(xml, PolyCatalog)

        # --- Deserialization ---
        # 1. PolyXML (Dataclass)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.deserialize(xml, PolyCatalog), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML",
                "Typed Dataclass",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 2. PolyXML (Pydantic v2)
        if num_items <= 2000:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: polyxml.deserialize(xml, PydanticCatalog), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "PolyXML (Pydantic)",
                    "Typed Pydantic",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 3. ElementTree (DOM)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: ET.fromstring(xml), size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "ElementTree",
                "Untyped DOM",
                "Deserialize",
                size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 4. defusedxml (Secure DOM)
        if HAS_DEFUSEDXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: defused_ET.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "defusedxml",
                    "Secure DOM",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 5. lxml (DOM)
        if HAS_LXML:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_ET.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml",
                    "Untyped DOM",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 6. lxml.objectify (C Dynamic Object)
        if HAS_LXML and lxml_objectify is not None:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_objectify.fromstring(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml.objectify",
                    "C Dynamic Object",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 7. xmltodict (Untyped Dict)
        if HAS_XMLTODICT:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xmltodict.parse(xml), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xmltodict",
                    "Untyped Dict",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 8. xsdata
        if HAS_XSDATA:
            xs_parser = XsXmlParser()
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xs_parser.from_bytes(xml, XsCatalog), size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xsdata",
                    "Typed Dataclass",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 9. pydantic-xml
        if HAS_PXML and num_items <= 1000:
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: PxmlCatalog.from_xml(xml), size, min(iterations, 10)
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "pydantic-xml",
                    "Typed Pydantic",
                    "Deserialize",
                    size,
                    min(iterations, 10),
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 10. declxml (Declarative Dict)
        if HAS_DECLXML and DECLXML_CATALOG_PROC is not None and num_items <= 2000:
            xml_str = xml.decode("utf-8")
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xml_decl.parse_from_string(DECLXML_CATALOG_PROC, xml_str),
                size,
                iterations,
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "declxml",
                    "Declarative Dict",
                    "Deserialize",
                    size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 11. untangle (Dynamic Object)
        if HAS_UNTANGLE and num_items <= 1000:
            xml_str = xml.decode("utf-8")
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: untangle.parse(xml_str), size, min(iterations, 10)
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "untangle",
                    "Dynamic Object",
                    "Deserialize",
                    size,
                    min(iterations, 10),
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # --- Serialization ---
        # 1. PolyXML (Dataclass)
        ser_xml = polyxml.serialize(poly_dc_instance)
        ser_size = len(ser_xml)
        min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
            lambda: polyxml.serialize(poly_dc_instance), ser_size, iterations
        )
        metrics.append(
            BenchmarkMetric(
                workload,
                "PolyXML",
                "Typed Dataclass",
                "Serialize",
                ser_size,
                iterations,
                min_ms,
                med_ms,
                mean_ms,
                p95_ms,
                thr,
                mem,
            )
        )

        # 2. lxml.objectify
        if HAS_LXML and lxml_objectify is not None:
            obj_inst = lxml_objectify.fromstring(xml)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: lxml_ET.tostring(obj_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "lxml.objectify",
                    "C Dynamic Object",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 3. xmltodict
        if HAS_XMLTODICT:
            d_inst = xmltodict.parse(xml)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xmltodict.unparse(d_inst), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xmltodict",
                    "Untyped Dict",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 4. xsdata
        if HAS_XSDATA:
            xs_serializer = XsXmlSerializer()
            xs_items = [
                XsCatalogItem(id=i, name=f"CatalogItem-{i}", price=i * 1.75)
                for i in range(num_items)
            ]
            xs_cat = XsCatalog(item=xs_items)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xs_serializer.render(xs_cat), ser_size, iterations
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "xsdata",
                    "Typed Dataclass",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        # 5. declxml
        if HAS_DECLXML and DECLXML_CATALOG_PROC is not None and num_items <= 2000:
            xml_str = xml.decode("utf-8")
            dec_inst = xml_decl.parse_from_string(DECLXML_CATALOG_PROC, xml_str)
            min_ms, med_ms, mean_ms, p95_ms, thr, mem = self.measure(
                lambda: xml_decl.serialize_to_string(DECLXML_CATALOG_PROC, dec_inst),
                ser_size,
                iterations,
            )
            metrics.append(
                BenchmarkMetric(
                    workload,
                    "declxml",
                    "Declarative Dict",
                    "Serialize",
                    ser_size,
                    iterations,
                    min_ms,
                    med_ms,
                    mean_ms,
                    p95_ms,
                    thr,
                    mem,
                )
            )

        return metrics

    def compute_speedups(self, metrics: list[BenchmarkMetric]) -> list[BenchmarkMetric]:
        """Compute relative speedup factor vs pure-python typed baseline (xsdata) or DOM."""
        grouped: dict[tuple[str, str], dict[str, BenchmarkMetric]] = {}
        for m in metrics:
            key = (m.workload, m.operation)
            grouped.setdefault(key, {})[m.engine] = m

        for engines in grouped.values():
            # Use xsdata as baseline if present; otherwise ElementTree
            baseline_metric = engines.get("xsdata") or engines.get("ElementTree")
            if baseline_metric and baseline_metric.min_ms > 0:
                base_time = baseline_metric.min_ms
                for m in engines.values():
                    if m.min_ms > 0:
                        m.speedup_vs_baseline = base_time / m.min_ms

        return metrics

    def display_rich_table(self, metrics: list[BenchmarkMetric]):
        """Render a formatted, rich console table of the benchmark results."""
        table = Table(
            title="⚡ PolyXML Multi-Parser Benchmark Results",
            show_header=True,
            header_style="bold cyan",
        )
        table.add_column("Workload", style="bold white", no_wrap=True)
        table.add_column("Engine", style="bold green")
        table.add_column("Category", style="dim")
        table.add_column("Op", style="magenta")
        table.add_column("Min Latency", justify="right")
        table.add_column("Median Latency", justify="right")
        table.add_column("Throughput", justify="right", style="bold yellow")
        table.add_column("Peak RAM", justify="right")
        table.add_column("Speedup vs Baseline", justify="right", style="bold cyan")

        for m in metrics:
            latency_str = (
                f"{m.min_ms * 1000.0:.1f} μs"
                if m.min_ms < 1.0
                else f"{m.min_ms:.2f} ms"
            )
            median_str = (
                f"{m.median_ms * 1000.0:.1f} μs"
                if m.median_ms < 1.0
                else f"{m.median_ms:.2f} ms"
            )
            thr_str = (
                f"{m.throughput_mbs:.1f} MB/s"
                if m.throughput_mbs > 0.05
                else "<0.1 MB/s"
            )
            mem_str = f"{m.peak_mem_kib:.1f} KiB"
            speedup_str = (
                f"{m.speedup_vs_baseline:.1f}x"
                if m.speedup_vs_baseline
                else "1.0x (Ref)"
            )

            table.add_row(
                m.workload,
                m.engine,
                m.target_type,
                m.operation,
                latency_str,
                median_str,
                thr_str,
                mem_str,
                speedup_str,
            )

        self.console.print(table)

    def export_markdown(self, metrics: list[BenchmarkMetric], path: str):
        """Export results as a GitHub Flavored Markdown table."""
        lines = [
            "# Benchmark Results",
            "",
            "| Workload | Engine | Type | Operation | Min Latency | Median Latency | Throughput | Peak RAM | Speedup |",
            "| :--- | :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |",
        ]
        for m in metrics:
            lat = (
                f"{m.min_ms * 1000.0:.1f} μs"
                if m.min_ms < 1.0
                else f"{m.min_ms:.2f} ms"
            )
            med = (
                f"{m.median_ms * 1000.0:.1f} μs"
                if m.median_ms < 1.0
                else f"{m.median_ms:.2f} ms"
            )
            thr = (
                f"{m.throughput_mbs:.1f} MB/s"
                if m.throughput_mbs > 0.05
                else "<0.1 MB/s"
            )
            mem = f"{m.peak_mem_kib:.1f} KiB"
            spd = f"{m.speedup_vs_baseline:.1f}x" if m.speedup_vs_baseline else "1.0x"
            lines.append(
                f"| {m.workload} | **{m.engine}** | {m.target_type} | {m.operation} | {lat} | {med} | {thr} | {mem} | {spd} |"
            )

        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")

    def export_json(self, metrics: list[BenchmarkMetric], path: str):
        """Export raw metrics as JSON."""
        data = [asdict(m) for m in metrics]
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
