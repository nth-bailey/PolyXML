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

#[test]
fn test_cli_typescript_generation() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("customer.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:crm">
            <xs:simpleType name="Status">
                <xs:restriction base="xs:string">
                    <xs:enumeration value="active"/>
                    <xs:enumeration value="suspended"/>
                </xs:restriction>
            </xs:simpleType>
            <xs:complexType name="Customer">
                <xs:sequence>
                    <xs:element name="name" type="xs:string"/>
                    <xs:element name="status" type="Status"/>
                    <xs:element name="tag" type="xs:string" minOccurs="0" maxOccurs="unbounded"/>
                </xs:sequence>
                <xs:attribute name="id" type="xs:int" use="required"/>
            </xs:complexType>
            <xs:element name="CustomerRecord" type="Customer"/>
        </xs:schema>"#,
    )
    .unwrap();

    // 1. Generate standard TypeScript
    let ts_out = dir.path().join("out_ts");
    let output_ts = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "ts",
            "--out",
            ts_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute typescript generate");

    assert!(output_ts.status.success());
    assert!(ts_out.join("index.ts").exists());
    let ts_code = fs::read_to_string(ts_out.join("customer.ts")).unwrap();
    assert!(ts_code.contains("export interface Customer {"));
    assert!(ts_code.contains("id: number;"));
    assert!(ts_code.contains("name: string;"));
    assert!(ts_code.contains("status: Status;"));
    assert!(ts_code.contains("tag: string[];"));
    assert!(ts_code.contains("export const Status = {"));
    assert!(ts_code.contains("export type Status = (typeof Status)[keyof typeof Status];"));
    assert!(ts_code.contains("export type CustomerRecord = Customer;"));

    // Verify TypeScript compiles cleanly with tsc --strict
    let tsc_check = Command::new("tsc")
        .args([
            "--noEmit",
            "--strict",
            "--target",
            "es2022",
            ts_out.join("customer.ts").to_str().unwrap(),
        ])
        .output();
    if let Ok(tsc_out) = tsc_check {
        assert!(
            tsc_out.status.success(),
            "tsc failed on customer.ts: {}\nstdout: {}",
            String::from_utf8_lossy(&tsc_out.stderr),
            String::from_utf8_lossy(&tsc_out.stdout)
        );
    }

    // 2. Generate TypeScript with Zod schemas
    let zod_out = dir.path().join("out_ts_zod");
    let output_zod = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "ts",
            "--zod",
            "--out",
            zod_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute typescript zod generate");

    assert!(output_zod.status.success());
    let zod_code = fs::read_to_string(zod_out.join("customer.ts")).unwrap();
    assert!(zod_code.contains("import { z } from \"zod\";"));
    assert!(zod_code.contains("export const StatusSchema = z.enum([\"active\", \"suspended\"]);"));
    assert!(zod_code.contains("export const CustomerSchema = z.object({"));
    assert!(zod_code.contains("id: z.number().int(),"));
    assert!(zod_code.contains("name: z.string(),"));
    assert!(zod_code.contains("status: StatusSchema,"));
    assert!(zod_code.contains("tag: z.array(z.string()),"));

    // Verify with tsc using ambient declaration for zod
    let stub_file = zod_out.join("zod_stub.d.ts");
    fs::write(
        &stub_file,
        "declare module \"zod\" { export const z: any; export namespace z { export type ZodType<T = any> = any; } }\n",
    )
    .unwrap();

    let tsc_zod_check = Command::new("tsc")
        .args([
            "--noEmit",
            "--strict",
            "--target",
            "es2022",
            stub_file.to_str().unwrap(),
            zod_out.join("customer.ts").to_str().unwrap(),
        ])
        .output();
    if let Ok(tsc_out) = tsc_zod_check {
        assert!(
            tsc_out.status.success(),
            "tsc failed on customer.ts with zod: {}\nstdout: {}",
            String::from_utf8_lossy(&tsc_out.stderr),
            String::from_utf8_lossy(&tsc_out.stdout)
        );
    }
}

#[test]
fn test_cli_java_generation() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("customer.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:crm">
            <xs:simpleType name="Status">
                <xs:restriction base="xs:string">
                    <xs:enumeration value="active"/>
                    <xs:enumeration value="suspended"/>
                </xs:restriction>
            </xs:simpleType>
            <xs:complexType name="Customer">
                <xs:sequence>
                    <xs:element name="name" type="xs:string"/>
                    <xs:element name="status" type="Status"/>
                    <xs:element name="tag" type="xs:string" minOccurs="0" maxOccurs="unbounded"/>
                </xs:sequence>
                <xs:attribute name="id" type="xs:int" use="required"/>
            </xs:complexType>
            <xs:element name="CustomerRecord" type="Customer"/>
        </xs:schema>"#,
    )
    .unwrap();

    let java_out = dir.path().join("out_java");
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "java",
            "--package",
            "com.enterprise.crm",
            "--out",
            java_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute java generate");

    assert!(output.status.success());
    assert!(java_out.join("Customer.java").exists());
    assert!(java_out.join("Status.java").exists());

    let customer_code = fs::read_to_string(java_out.join("Customer.java")).unwrap();
    assert!(customer_code.contains("package com.enterprise.crm;"));
    assert!(customer_code.contains("public record Customer("));
    assert!(customer_code.contains("int id"));
    assert!(customer_code.contains("String name,"));
    assert!(customer_code.contains("Status status,"));
    assert!(customer_code.contains("java.util.List<String> tag"));

    let status_code = fs::read_to_string(java_out.join("Status.java")).unwrap();
    assert!(status_code.contains("package com.enterprise.crm;"));
    assert!(status_code.contains("public enum Status {"));
    assert!(status_code.contains("ACTIVE(\"active\"),"));
    assert!(status_code.contains("SUSPENDED(\"suspended\");"));

    // Verify Java compilation with javac -Werror
    let javac_check = Command::new("javac")
        .args([
            "-Werror",
            java_out.join("Status.java").to_str().unwrap(),
            java_out.join("Customer.java").to_str().unwrap(),
        ])
        .output();

    if let Ok(javac_out) = javac_check {
        assert!(
            javac_out.status.success(),
            "javac failed on generated Java 21 files: {}\nstdout: {}",
            String::from_utf8_lossy(&javac_out.stderr),
            String::from_utf8_lossy(&javac_out.stdout)
        );
    }
}

#[test]
fn test_cli_cpp_generation() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("crm.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0"?>
        <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="urn:enterprise:crm">
            <xs:simpleType name="Status">
                <xs:restriction base="xs:string">
                    <xs:enumeration value="active"/>
                    <xs:enumeration value="suspended"/>
                </xs:restriction>
            </xs:simpleType>
            <xs:complexType name="Customer">
                <xs:sequence>
                    <xs:element name="name" type="xs:string"/>
                    <xs:element name="email" type="xs:string" minOccurs="0"/>
                    <xs:element name="status" type="Status"/>
                    <xs:element name="tag" type="xs:string" minOccurs="0" maxOccurs="unbounded"/>
                </xs:sequence>
                <xs:attribute name="id" type="xs:int" use="required"/>
            </xs:complexType>
            <xs:element name="CustomerRecord" type="Customer"/>
        </xs:schema>"#,
    )
    .unwrap();

    let cpp_out = dir.path().join("out_cpp");
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "cpp",
            "--namespace",
            "enterprise::crm",
            "--out",
            cpp_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
            "--format",
        ])
        .output()
        .expect("Failed to execute cpp generate");

    assert!(
        output.status.success(),
        "polyxml generate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let header_file = cpp_out.join("crm.hpp");
    assert!(header_file.exists(), "crm.hpp was not generated");

    let cpp_code = fs::read_to_string(&header_file).unwrap();
    assert!(cpp_code.contains("namespace enterprise::crm {"));
    assert!(cpp_code.contains("struct Customer {"));
    assert!(cpp_code.contains("std::string name"));
    assert!(cpp_code.contains("std::optional<std::string> email"));
    assert!(cpp_code.contains("Status status"));
    assert!(cpp_code.contains("std::vector<std::string> tag"));
    assert!(cpp_code.contains("std::int32_t id"));
    assert!(cpp_code.contains("enum class Status {"));
    assert!(cpp_code.contains("Active,"));
    assert!(cpp_code.contains("Suspended,"));
    assert!(cpp_code.contains("using CustomerRecord = Customer;"));
    assert!(cpp_code.contains("operator==") && cpp_code.contains("default"));

    // Verify C++20 compilation and execution with g++
    let driver_cpp = dir.path().join("driver.cpp");
    fs::write(
        &driver_cpp,
        r#"
#include "crm.hpp"
#include <cassert>
#include <iostream>

int main() {
    using namespace enterprise::crm;

    Customer c1{
        .name = "Acme Corp",
        .email = "info@acme.com",
        .status = Status::Active,
        .tag = {"enterprise", "partner"},
        .id = 100
    };

    Customer c2{
        .name = "Acme Corp",
        .email = "info@acme.com",
        .status = Status::Active,
        .tag = {"enterprise", "partner"},
        .id = 100
    };

    Customer c3{
        .name = "Beta LLC",
        .email = std::nullopt,
        .status = Status::Suspended,
        .tag = {},
        .id = 101
    };

    assert(c1 == c2);
    assert(!(c1 == c3));
    assert(to_string(Status::Active) == "active");
    assert(to_string(Status::Suspended) == "suspended");

    std::cout << "E2E C++20 driver passed!" << std::endl;
    return 0;
}
"#,
    )
    .unwrap();

    let out_bin = dir.path().join("driver_bin");
    let compile_status = Command::new("g++")
        .args([
            "-std=c++20",
            "-Wall",
            "-Wextra",
            "-Wpedantic",
            "-Werror",
            "-I",
            cpp_out.to_str().unwrap(),
            driver_cpp.to_str().unwrap(),
            "-o",
            out_bin.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute g++");

    assert!(
        compile_status.success(),
        "g++ compilation of generated crm.hpp failed"
    );

    let run_status = Command::new(&out_bin)
        .status()
        .expect("Failed to run compiled C++ binary");
    assert!(run_status.success(), "C++ test driver failed execution");
}

#[test]
fn test_cli_go_generation() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("crm.xsd");
    fs::write(
        &schema_file,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="https://example.com/crm">
    <xs:simpleType name="AccountTier">
        <xs:restriction base="xs:string">
            <xs:enumeration value="standard"/>
            <xs:enumeration value="premium"/>
            <xs:enumeration value="enterprise"/>
        </xs:restriction>
    </xs:simpleType>

    <xs:complexType name="Account">
        <xs:sequence>
            <xs:element name="name" type="xs:string"/>
            <xs:element name="tier" type="AccountTier"/>
            <xs:element name="balance" type="xs:decimal"/>
            <xs:element name="alias" type="xs:string" minOccurs="0"/>
            <xs:element name="tag" type="xs:string" minOccurs="0" maxOccurs="unbounded"/>
        </xs:sequence>
        <xs:attribute name="id" type="xs:int" use="required"/>
    </xs:complexType>

    <xs:element name="AccountRecord" type="Account"/>
</xs:schema>"#,
    )
    .unwrap();

    let go_out = dir.path().join("out_go");
    let output = Command::new(env!("CARGO_BIN_EXE_polyxml"))
        .args([
            "generate",
            "--lang",
            "go",
            "--package",
            "crm",
            "--out",
            go_out.to_str().unwrap(),
            schema_file.to_str().unwrap(),
            "--format",
        ])
        .output()
        .expect("Failed to execute Go generate");

    assert!(
        output.status.success(),
        "CLI generate --lang go failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated_file = go_out.join("crm.go");
    assert!(generated_file.exists(), "crm.go was not created");
    let go_code = fs::read_to_string(&generated_file).unwrap();

    assert!(go_code.contains("package crm"));
    assert!(go_code.contains("type AccountTier string"));
    assert!(go_code.contains("AccountTierStandard"));
    assert!(go_code.contains("\"standard\""));
    assert!(go_code.contains("func (e AccountTier) IsValid() bool"));
    assert!(go_code.contains("type Account struct {"));
    assert!(go_code.contains("XMLName xml.Name"));
    assert!(go_code.contains("Name") && go_code.contains("`xml:\"name\"`"));
    assert!(
        go_code.contains("Tier")
            && go_code.contains("AccountTier")
            && go_code.contains("`xml:\"tier\"`")
    );
    assert!(
        go_code.contains("Balance")
            && go_code.contains("float64")
            && go_code.contains("`xml:\"balance\"`")
    );
    assert!(
        go_code.contains("Alias")
            && go_code.contains("*string")
            && go_code.contains("`xml:\"alias,omitempty\"`")
    );
    assert!(
        go_code.contains("Tag")
            && go_code.contains("[]string")
            && go_code.contains("`xml:\"tag\"`")
    );
    assert!(
        go_code.contains("ID")
            && go_code.contains("int32")
            && go_code.contains("`xml:\"id,attr\"`")
    );
    assert!(go_code.contains("func (s Account) Validate() error"));

    // Write a Go test driver to verify with `go test` and `go vet`
    let driver_go = go_out.join("crm_test.go");
    fs::write(
        &driver_go,
        r#"package crm

import (
    "encoding/xml"
    "testing"
)

func TestAccountE2E(t *testing.T) {
    alias := "AcmeMain"
    acc := Account{
        Name:    "Acme Corp",
        Tier:    AccountTierPremium,
        Balance: 1250.50,
        Alias:   &alias,
        Tag:     []string{"b2b", "strategic"},
        ID:      1001,
    }

    if !acc.Tier.IsValid() {
        t.Fatalf("expected tier to be valid")
    }

    data, err := xml.MarshalIndent(acc, "", "  ")
    if err != nil {
        t.Fatalf("xml.Marshal failed: %v", err)
    }

    var decoded Account
    if err := xml.Unmarshal(data, &decoded); err != nil {
        t.Fatalf("xml.Unmarshal failed: %v", err)
    }

    if decoded.Name != "Acme Corp" || decoded.Tier != AccountTierPremium || decoded.ID != 1001 {
        t.Fatalf("mismatched decoded values: %+v", decoded)
    }
    if decoded.Alias == nil || *decoded.Alias != "AcmeMain" {
        t.Fatalf("mismatched alias: %+v", decoded.Alias)
    }
    if len(decoded.Tag) != 2 || decoded.Tag[0] != "b2b" {
        t.Fatalf("mismatched tags: %+v", decoded.Tag)
    }

    if err := decoded.Validate(); err != nil {
        t.Fatalf("validation failed: %v", err)
    }
}
"#,
    )
    .unwrap();

    let init_status = Command::new("go")
        .args(["mod", "init", "crm"])
        .current_dir(&go_out)
        .status()
        .expect("Failed to run go mod init");
    assert!(init_status.success(), "go mod init failed");

    let vet_status = Command::new("go")
        .args(["vet", "."])
        .current_dir(&go_out)
        .status()
        .expect("Failed to run go vet");
    assert!(vet_status.success(), "go vet failed on generated Go models");

    let test_status = Command::new("go")
        .args(["test", "-v", "."])
        .current_dir(&go_out)
        .status()
        .expect("Failed to run go test");
    assert!(
        test_status.success(),
        "go test failed on generated Go models"
    );
}
