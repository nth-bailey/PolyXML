use std::fs;
use std::process::Command;
use tempfile::tempdir;

use polyxml_cli::config::WorkspaceManifest;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Polyglot XML schema compiler"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("validate"));
}

#[test]
fn test_cli_generate_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["generate", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--lang"));
    assert!(stdout.contains("--out"));
    assert!(stdout.contains("--strict-facets"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--format"));
}

#[test]
fn test_cli_validate_valid_and_invalid() {
    let dir = tempdir().unwrap();
    let valid_xsd = dir.path().join("test.xsd");
    fs::write(
        &valid_xsd,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:test">
            <xs:element name="Hello" type="xs:string"/>
        </xs:schema>"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["validate", valid_xsd.to_str().unwrap()])
        .output()
        .expect("Failed to execute validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Valid schema"));
    assert!(stdout.contains("urn:test"));

    // Test non-existent schema
    let missing_xsd = dir.path().join("missing.xsd");
    let bad_output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["validate", missing_xsd.to_str().unwrap()])
        .output()
        .expect("Failed to execute validate");

    assert!(!bad_output.status.success());
}

#[test]
fn test_cli_generate_dry_run() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("order.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:orders">
            <xs:complexType name="Order">
                <xs:sequence>
                    <xs:element name="Id" type="xs:string"/>
                    <xs:element name="Amount" type="xs:decimal"/>
                </xs:sequence>
            </xs:complexType>
            <xs:element name="PurchaseOrder" type="Order"/>
        </xs:schema>"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["generate", "--dry-run", schema_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute generate --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run completed successfully. No files written."));
    assert!(stdout.contains("Namespace: urn:orders"));
    assert!(stdout.contains("Root elements: 1"));
}

#[test]
fn test_cli_generate_multi_lang() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("sample.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:sample">
            <xs:complexType name="SampleItem">
                <xs:sequence>
                    <xs:element name="Name" type="xs:string"/>
                </xs:sequence>
            </xs:complexType>
        </xs:schema>"#,
    )
    .unwrap();

    let out_dir = dir.path().join("out");

    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "python",
            "--lang",
            "rust",
            "--out",
            out_dir.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute generate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Code generation complete."));

    assert!(out_dir.join("python/sample.py").exists());
    assert!(out_dir.join("rust/sample.rs").exists());
}

#[test]
fn test_polyxml_toml_manifest_build() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("contract.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:corp">
            <xs:element name="Agreement" type="xs:string"/>
        </xs:schema>"#,
    )
    .unwrap();

    let manifest_file = dir.path().join("polyxml.toml");
    let manifest_content = r#"
[workspace]
name = "contracts"
schemas = ["*.xsd"]
output_base_dir = "./dist"

[[generate]]
target = "python"
output = "py_models"
backend = "pydantic-v2"

[codegen.rust]
enabled = true
output = "rs_models"
zero_copy = true
"#;
    fs::write(&manifest_file, manifest_content).unwrap();

    // Test config parsing unit
    let parsed = WorkspaceManifest::from_file(&manifest_file).unwrap();
    assert_eq!(
        parsed.workspace.as_ref().unwrap().name.as_deref(),
        Some("contracts")
    );
    let targets = parsed.resolved_targets();
    assert_eq!(targets.len(), 2);
    let schemas = parsed.expand_schemas(dir.path()).unwrap();
    assert_eq!(schemas.len(), 1);

    // Test CLI build command
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args(["build", "--config", manifest_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute build");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build finished successfully."));
    assert!(dir.path().join("dist/py_models/contract.py").exists());
    assert!(dir.path().join("dist/rs_models/contract.rs").exists());
}

#[test]
fn test_cli_generate_python_backends() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("invoice.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="https://example.com/invoice">
            <xs:simpleType name="InvoiceCode">
                <xs:restriction base="xs:string">
                    <xs:minLength value="5"/>
                    <xs:maxLength value="10"/>
                </xs:restriction>
            </xs:simpleType>
            <xs:complexType name="Invoice">
                <xs:sequence>
                    <xs:element name="Code" type="InvoiceCode"/>
                    <xs:element name="Total" type="xs:decimal"/>
                    <xs:element name="Note" type="xs:string" minOccurs="0" nillable="true"/>
                </xs:sequence>
                <xs:attribute name="id" type="xs:int" use="required"/>
            </xs:complexType>
            <xs:element name="InvoiceDoc" type="Invoice"/>
        </xs:schema>"#,
    )
    .unwrap();

    // 1. Generate with dataclass backend
    let dc_out = dir.path().join("out_dataclass");
    let output_dc = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "python",
            "--backend",
            "dataclass",
            "--out",
            dc_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute dataclass generate");

    assert!(output_dc.status.success());
    let dc_py = fs::read_to_string(dc_out.join("invoice.py")).unwrap();
    assert!(dc_py.contains("@dataclass(slots=True, kw_only=True)"));
    assert!(dc_py.contains("class Invoice:"));
    assert!(dc_py.contains("code: InvoiceCode = field("));
    assert!(dc_py.contains("total: Decimal = field("));
    assert!(dc_py.contains("note: str | None = field(default=None"));
    assert!(dc_py.contains("id: int = field("));
    assert!(dc_py.contains("type InvoiceDoc = Invoice"));

    // 2. Generate with pydantic backend
    let pydantic_out = dir.path().join("out_pydantic");
    let output_pydantic = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "python",
            "--backend",
            "pydantic",
            "--out",
            pydantic_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute pydantic generate");

    assert!(output_pydantic.status.success());
    let pyd_py = fs::read_to_string(pydantic_out.join("invoice.py")).unwrap();
    assert!(pyd_py.contains("class Invoice(BaseModel):"));
    assert!(pyd_py.contains("model_config = ConfigDict(defer_build=True, populate_by_name=True)"));
    assert!(
        pyd_py.contains("type InvoiceCode = Annotated[str, Field(min_length=5, max_length=10)]")
    );
    assert!(pyd_py.contains("code: InvoiceCode = Field(..., json_schema_extra={\"type\": \"Element\", \"name\": \"Code\", \"namespace\": \"https://example.com/invoice\"})"));
    assert!(pyd_py.contains("total: Decimal = Field(..., json_schema_extra={\"type\": \"Element\", \"name\": \"Total\", \"namespace\": \"https://example.com/invoice\"})"));
    assert!(pyd_py.contains("note: str | None = Field(default=None, json_schema_extra={\"type\": \"Element\", \"name\": \"Note\", \"namespace\": \"https://example.com/invoice\", \"nillable\": True})"));
}

#[test]
fn test_cli_generate_rust_zero_copy_and_owned() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("customer.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:customers">
            <xs:simpleType name="Status">
                <xs:restriction base="xs:string">
                    <xs:enumeration value="Active"/>
                    <xs:enumeration value="Suspended"/>
                </xs:restriction>
            </xs:simpleType>
            <xs:complexType name="Customer">
                <xs:sequence>
                    <xs:element name="Name" type="xs:string"/>
                    <xs:element name="Status" type="Status"/>
                    <xs:element name="Balance" type="xs:decimal"/>
                </xs:sequence>
                <xs:attribute name="id" type="xs:int" use="required"/>
            </xs:complexType>
            <xs:element name="CustomerRecord" type="Customer"/>
        </xs:schema>"#,
    )
    .unwrap();

    // 1. Generate zero-copy Rust
    let zc_out = dir.path().join("out_rust_zc");
    let output_zc = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "rust",
            "--zero-copy",
            "--out",
            zc_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute rust zero-copy generate");

    assert!(output_zc.status.success());
    assert!(zc_out.join("mod.rs").exists());
    let zc_rs = fs::read_to_string(zc_out.join("customer.rs")).unwrap();
    assert!(zc_rs.contains("use std::borrow::Cow;"));
    assert!(zc_rs.contains("pub struct Customer<'a>"));
    assert!(zc_rs.contains("pub name: Cow<'a, str>"));
    assert!(zc_rs.contains("pub enum Status"));
    assert!(zc_rs.contains("impl Status"));
    assert!(zc_rs.contains("pub type CustomerRecord<'a> = Customer<'a>;"));

    // 2. Generate owned Rust
    let owned_out = dir.path().join("out_rust_owned");
    let output_owned = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "rust",
            "--zero-copy",
            "false",
            "--out",
            owned_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute rust owned generate");

    assert!(output_owned.status.success());
    let owned_rs = fs::read_to_string(owned_out.join("customer.rs")).unwrap();
    assert!(owned_rs.contains("pub struct Customer {"));
    assert!(owned_rs.contains("pub name: String"));
    assert!(!owned_rs.contains("Cow<'a"));
    assert!(owned_rs.contains("pub type CustomerRecord = Customer;"));
}
