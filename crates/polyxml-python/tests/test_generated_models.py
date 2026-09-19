import importlib.util
import pathlib
import shutil
import subprocess
import sys
import tempfile
from decimal import Decimal

import pytest

import polyxml

SAMPLE_XSD = """<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="https://example.com/warehouse"
           xmlns="https://example.com/warehouse"
           elementFormDefault="qualified">

    <xs:simpleType name="PartStatus">
        <xs:restriction base="xs:string">
            <xs:enumeration value="in-stock"/>
            <xs:enumeration value="back-ordered"/>
            <xs:enumeration value="discontinued"/>
        </xs:restriction>
    </xs:simpleType>

    <xs:simpleType name="PartSku">
        <xs:restriction base="xs:string">
            <xs:minLength value="4"/>
            <xs:maxLength value="12"/>
            <xs:pattern value="[A-Z]{3}-[0-9]{4}"/>
        </xs:restriction>
    </xs:simpleType>

    <xs:simpleType name="Quantity">
        <xs:restriction base="xs:integer">
            <xs:minInclusive value="0"/>
            <xs:maxInclusive value="9999"/>
        </xs:restriction>
    </xs:simpleType>

    <xs:complexType name="Dimension">
        <xs:sequence>
            <xs:element name="width" type="xs:double"/>
            <xs:element name="height" type="xs:double"/>
            <xs:element name="depth" type="xs:double"/>
        </xs:sequence>
    </xs:complexType>

    <xs:complexType name="PartItem">
        <xs:sequence>
            <xs:element name="sku" type="PartSku"/>
            <xs:element name="name" type="xs:string"/>
            <xs:element name="status" type="PartStatus"/>
            <xs:element name="quantity" type="Quantity"/>
            <xs:element name="price" type="xs:decimal"/>
            <xs:element name="description" type="xs:string" minOccurs="0" nillable="true"/>
            <xs:element name="dimension" type="Dimension" minOccurs="0"/>
            <xs:element name="tags" type="xs:string" minOccurs="0" maxOccurs="unbounded"/>
        </xs:sequence>
        <xs:attribute name="id" type="xs:int" use="required"/>
        <xs:attribute name="active" type="xs:boolean" default="true"/>
    </xs:complexType>

    <xs:complexType name="Inventory">
        <xs:sequence>
            <xs:element name="warehouseName" type="xs:string"/>
            <xs:element name="items" type="PartItem" minOccurs="0" maxOccurs="unbounded"/>
        </xs:sequence>
    </xs:complexType>

    <xs:element name="WarehouseInventory" type="Inventory"/>
</xs:schema>
"""

SAMPLE_XML = b"""<Inventory xmlns="https://example.com/warehouse">
    <warehouseName>Central Distribution</warehouseName>
    <items id="101" active="true">
        <sku>ENG-1234</sku>
        <name>Gearbox Module</name>
        <status>in-stock</status>
        <quantity>45</quantity>
        <price>499.95</price>
        <description>Heavy duty aerospace gearbox</description>
        <dimension>
            <width>12.5</width>
            <height>8.0</height>
            <depth>15.2</depth>
        </dimension>
        <tags>aviation</tags>
        <tags>powertrain</tags>
    </items>
    <items id="102" active="false">
        <sku>HYD-5678</sku>
        <name>Hydraulic Valve</name>
        <status>back-ordered</status>
        <quantity>0</quantity>
        <price>89.50</price>
        <description nil="true"/>
    </items>
</Inventory>
"""


def _load_module_from_file(module_name: str, file_path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(module_name, file_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def _get_polyxml_bin() -> pathlib.Path:
    repo_root = pathlib.Path(__file__).parent.parent.parent.parent
    exe_name = "polyxml.exe" if sys.platform == "win32" else "polyxml"
    candidates = [
        repo_root / "target" / "debug" / exe_name,
        repo_root / "target" / "release" / exe_name,
    ]
    for c in candidates:
        if c.exists():
            return c

    which_path = shutil.which(exe_name) or shutil.which("polyxml")
    if which_path:
        return pathlib.Path(which_path)

    # Attempt on-the-fly compilation via cargo if not found
    subprocess.run(
        ["cargo", "build", "-p", "polyxml-cli"],
        cwd=repo_root,
        check=True,
        capture_output=True,
    )
    for c in candidates:
        if c.exists():
            return c

    raise FileNotFoundError(f"polyxml CLI binary could not be found or built at {candidates}")


@pytest.fixture(scope="module")
def generated_models():
    """Generates both Dataclass and Pydantic models from SAMPLE_XSD using polyxml CLI,
    verifying ruff and pyright compliance."""
    polyxml_bin = _get_polyxml_bin()

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = pathlib.Path(tmpdir)
        xsd_file = tmp_path / "warehouse.xsd"
        xsd_file.write_text(SAMPLE_XSD)

        # 1. Generate Dataclass models
        dc_dir = tmp_path / "gen_dc"
        dc_dir.mkdir()
        res_dc = subprocess.run(
            [
                str(polyxml_bin),
                "generate",
                "--lang",
                "python",
                "--backend",
                "dataclass",
                "--out",
                str(dc_dir),
                str(xsd_file),
                "--format",
            ],
            capture_output=True,
            text=True,
        )
        assert res_dc.returncode == 0, f"Dataclass codegen failed: {res_dc.stderr}"
        dc_file = dc_dir / "warehouse.py"
        assert dc_file.exists()

        # 2. Generate Pydantic models
        pyd_dir = tmp_path / "gen_pyd"
        pyd_dir.mkdir()
        res_pyd = subprocess.run(
            [
                str(polyxml_bin),
                "generate",
                "--lang",
                "python",
                "--backend",
                "pydantic",
                "--out",
                str(pyd_dir),
                str(xsd_file),
                "--format",
            ],
            capture_output=True,
            text=True,
        )
        assert res_pyd.returncode == 0, f"Pydantic codegen failed: {res_pyd.stderr}"
        pyd_file = pyd_dir / "warehouse.py"
        assert pyd_file.exists()

        # 3. Verify ruff check on both
        ruff_check_dc = subprocess.run(
            ["ruff", "check", str(dc_dir)], capture_output=True, text=True
        )
        assert ruff_check_dc.returncode == 0, (
            f"Ruff check failed on dataclasses: {ruff_check_dc.stdout}\n{ruff_check_dc.stderr}"
        )

        ruff_check_pyd = subprocess.run(
            ["ruff", "check", str(pyd_dir)], capture_output=True, text=True
        )
        assert ruff_check_pyd.returncode == 0, (
            f"Ruff check failed on pydantic: {ruff_check_pyd.stdout}\n{ruff_check_pyd.stderr}"
        )

        # 4. Verify pyright static type checker on both
        pyright_dc = subprocess.run(["pyright", str(dc_file)], capture_output=True, text=True)
        assert pyright_dc.returncode == 0, f"Pyright failed on dataclasses: {pyright_dc.stdout}"

        pyright_pyd = subprocess.run(["pyright", str(pyd_file)], capture_output=True, text=True)
        assert pyright_pyd.returncode == 0, f"Pyright failed on pydantic: {pyright_pyd.stdout}"

        # 5. Load modules dynamically
        mod_dc = _load_module_from_file("gen_warehouse_dc", dc_file)
        mod_pyd = _load_module_from_file("gen_warehouse_pyd", pyd_file)

        yield {"dataclass": mod_dc, "pydantic": mod_pyd}


def test_generated_dataclass_deserialization_and_serialization(generated_models):
    mod = generated_models["dataclass"]

    # Verify Enum
    assert hasattr(mod, "PartStatus")
    assert mod.PartStatus.IN_STOCK.value == "in-stock"
    assert mod.PartStatus.BACK_ORDERED.value == "back-ordered"

    # Deserialize
    inv = polyxml.deserialize(SAMPLE_XML, mod.Inventory)
    assert inv.warehouse_name == "Central Distribution"
    assert len(inv.items) == 2

    # Check item 0
    item0 = inv.items[0]
    assert item0.id == 101
    assert item0.active is True
    assert item0.sku == "ENG-1234"
    assert item0.name == "Gearbox Module"
    assert item0.status == mod.PartStatus.IN_STOCK
    assert item0.quantity == 45
    assert item0.price == Decimal("499.95")
    assert item0.description == "Heavy duty aerospace gearbox"
    assert item0.dimension is not None
    assert item0.dimension.width == 12.5
    assert item0.dimension.height == 8.0
    assert item0.dimension.depth == 15.2
    assert item0.tags == ["aviation", "powertrain"]

    # Check item 1
    item1 = inv.items[1]
    assert item1.id == 102
    assert item1.active is False
    assert item1.sku == "HYD-5678"
    assert item1.status == mod.PartStatus.BACK_ORDERED
    assert item1.quantity == 0
    assert item1.price == Decimal("89.50")
    assert item1.dimension is None

    # Reserialize
    xml_out = polyxml.serialize(inv)
    assert b"Central Distribution" in xml_out
    assert b"ENG-1234" in xml_out
    assert b"HYD-5678" in xml_out
    assert b"aviation" in xml_out


def test_generated_pydantic_deserialization_and_serialization(generated_models):
    mod = generated_models["pydantic"]

    assert hasattr(mod, "PartStatus")
    assert hasattr(mod, "Inventory")
    assert hasattr(mod, "PartItem")

    # Deserialize into Pydantic model
    inv = polyxml.deserialize(SAMPLE_XML, mod.Inventory)
    assert inv.warehouse_name == "Central Distribution"
    assert len(inv.items) == 2

    item0 = inv.items[0]
    assert item0.id == 101
    assert item0.sku == "ENG-1234"
    assert item0.price == Decimal("499.95")
    assert item0.quantity == 45
    assert item0.tags == ["aviation", "powertrain"]

    # Test Pydantic model dump
    dumped = inv.model_dump()
    assert dumped["warehouse_name"] == "Central Distribution"
    assert len(dumped["items"]) == 2

    # Reserialize back to XML
    xml_out = polyxml.serialize(inv)
    assert b"Central Distribution" in xml_out
    assert b"ENG-1234" in xml_out

    # Test inherent codecs on Pydantic model: from_xml and to_xml
    inv_codec = mod.Inventory.from_xml(SAMPLE_XML)
    assert inv_codec.warehouse_name == "Central Distribution"
    assert len(inv_codec.items) == 2

    # String input support
    inv_str = mod.Inventory.from_xml(SAMPLE_XML.decode("utf-8"))
    assert inv_str.warehouse_name == "Central Distribution"

    # to_xml support with indentation
    xml_codec_bytes = inv_codec.to_xml(indent=2)
    assert b"Central Distribution" in xml_codec_bytes
    assert b"\n" in xml_codec_bytes


def test_generated_dataclass_codecs(generated_models):
    mod = generated_models["dataclass"]

    # Inherent from_xml on Dataclass model
    inv = mod.Inventory.from_xml(SAMPLE_XML)
    assert inv.warehouse_name == "Central Distribution"
    assert len(inv.items) == 2
    assert inv.items[0].sku == "ENG-1234"

    # String input
    inv_str = mod.Inventory.from_xml(SAMPLE_XML.decode("utf-8"))
    assert inv_str.warehouse_name == "Central Distribution"

    # Inherent to_xml
    xml_bytes = inv.to_xml(indent=4)
    assert b"Central Distribution" in xml_bytes
    assert b"ENG-1234" in xml_bytes


def test_codecs_flag_disabled():
    polyxml_bin = _get_polyxml_bin()

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = pathlib.Path(tmpdir)
        xsd_file = tmp_path / "warehouse.xsd"
        xsd_file.write_text(SAMPLE_XSD)

        out_dir = tmp_path / "no_codecs"
        out_dir.mkdir()
        res = subprocess.run(
            [
                str(polyxml_bin),
                "generate",
                "--lang",
                "python",
                "--codecs",
                "false",
                "--out",
                str(out_dir),
                str(xsd_file),
            ],
            capture_output=True,
            text=True,
        )
        assert res.returncode == 0
        mod = _load_module_from_file("gen_no_codecs", out_dir / "warehouse.py")
        assert not hasattr(mod.Inventory, "from_xml")
        assert not hasattr(mod.Inventory, "to_xml")
