use std::fs;
use std::process::Command;

use polyxml::codegen::csharp::{
    to_csharp_namespace, to_csharp_param_name, to_csharp_property_name, to_csharp_type_name,
    to_csharp_variant_name, CSharpCodegen, CSharpOptions, CSharpRecordKind,
};
use polyxml::ir::{
    Cardinality, EnumDef, EnumValue, FieldDef, FieldKind, PrimitiveType, QName, RestrictionFacets,
    SchemaIR, StructDef, TypeDef, TypeRef, UnionBranch, UnionDef,
};
use tempfile::tempdir;

fn dotnet_command() -> Command {
    let mut cmd = Command::new("dotnet");
    cmd.env("DOTNET_NOLOGO", "1");
    cmd.env("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
    cmd.env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1");
    cmd
}

#[test]
fn test_csharp_sanitization() {
    assert_eq!(to_csharp_type_name("event"), "Event");
    assert_eq!(to_csharp_type_name("class"), "Class");
    assert_eq!(to_csharp_type_name("3d_point"), "Type3dPoint");
    assert_eq!(to_csharp_type_name("customer_account"), "CustomerAccount");

    // Property name sanitization avoiding collision with enclosing type (CS0542)
    assert_eq!(
        to_csharp_property_name("customer", Some("Customer")),
        "CustomerValue"
    );
    assert_eq!(to_csharp_property_name("name", Some("Customer")), "Name");

    // Keyword parameter name sanitization
    assert_eq!(to_csharp_param_name("event"), "@event");
    assert_eq!(to_csharp_param_name("params"), "@params");
    assert_eq!(to_csharp_param_name("first_name"), "firstName");

    // Variant names
    assert_eq!(to_csharp_variant_name("in_progress"), "InProgress");
    assert_eq!(to_csharp_variant_name("10_days"), "V10Days");

    // Dotted namespaces
    assert_eq!(
        to_csharp_namespace("com.example.crm-models"),
        "Com.Example.CrmModels"
    );
}

#[test]
fn test_csharp_records_and_enums_generation() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/crm");

    // Enum: OrderStatus
    let enum_qname = QName::new(Some("https://example.com/crm"), "OrderStatus");
    ir.add_type(TypeDef::Enum(EnumDef {
        qname: enum_qname.clone(),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        variants: vec![
            EnumValue {
                name: "pending".into(),
                value: "pending".into(),
                documentation: Some("Pending processing".into()),
            },
            EnumValue {
                name: "in-progress".into(),
                value: "in-progress".into(),
                documentation: None,
            },
            EnumValue {
                name: "completed".into(),
                value: "completed".into(),
                documentation: None,
            },
        ],
        documentation: Some("Order processing state".into()),
    }));

    // Struct: Customer
    let customer_qname = QName::new(Some("https://example.com/crm"), "Customer");
    let facets = RestrictionFacets {
        min_length: Some(3),
        ..Default::default()
    };

    ir.add_type(TypeDef::Struct(StructDef {
        qname: customer_qname.clone(),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef {
                name: "id".into(),
                xml_name: "id".into(),
                namespace: None,
                kind: FieldKind::Attribute,
                type_ref: TypeRef::Primitive(PrimitiveType::Int),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "name".into(),
                xml_name: "name".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: Some(facets),
                is_cycle_cut: false,
            },
            FieldDef {
                name: "email".into(),
                xml_name: "email".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::optional_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "tag".into(),
                xml_name: "tag".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::unbounded(0),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "status".into(),
                xml_name: "status".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Named(enum_qname),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
        ],
        documentation: Some("Customer record definition".into()),
    }));

    let options = CSharpOptions {
        namespace: "Crm.Models".to_string(),
        emit_xml_attributes: true,
        emit_json_attributes: true,
        emit_validation: true,
        record_kind: CSharpRecordKind::Class,
        use_file_scoped_namespaces: true,
        emit_root_records: true,
        ..Default::default()
    };

    let codegen = CSharpCodegen::new(options);
    let cs_code = codegen.generate_module(&ir);

    assert!(cs_code.contains("using System.Text.Json.Serialization;"));
    assert!(cs_code.contains("namespace Crm.Models;"));
    assert!(cs_code.contains("[JsonConverter(typeof(JsonStringEnumConverter<OrderStatus>))]"));
    assert!(cs_code.contains("public enum OrderStatus"));
    assert!(cs_code.contains("[XmlEnum(\"pending\")]"));
    assert!(cs_code.contains("Pending,"));
    assert!(cs_code.contains("public static bool IsValid(this OrderStatus value)"));
    assert!(cs_code.contains("public static string ToXmlValue(this OrderStatus value)"));
    assert!(cs_code.contains("public record Customer("));
    assert!(cs_code.contains("[property: XmlAttribute(\"id\"), JsonPropertyName(\"id\")] int Id,"));
    assert!(cs_code
        .contains("[property: XmlElement(\"name\"), JsonPropertyName(\"name\")] string Name,"));
    assert!(cs_code
        .contains("[property: XmlElement(\"email\"), JsonPropertyName(\"email\")] string? Email,"));
    assert!(cs_code
        .contains("[property: XmlElement(\"tag\"), JsonPropertyName(\"tag\")] List<string>? Tag,"));
    assert!(cs_code.contains(
        "[property: XmlElement(\"status\"), JsonPropertyName(\"status\")] OrderStatus Status"
    ));
    assert!(cs_code.contains("public Customer() : this("));
    assert!(cs_code.contains("IValidatableObject"));

    // Verify .NET compilation, serialization roundtrip, and validation
    let temp = tempdir().unwrap();
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>"#;
    fs::write(temp.path().join("CrmApp.csproj"), csproj).unwrap();

    let models_cs = temp.path().join("Models.cs");
    fs::write(&models_cs, &cs_code).unwrap();

    let program_cs = temp.path().join("Program.cs");
    fs::write(
        &program_cs,
        r#"using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.IO;
using System.Text.Json;
using System.Xml.Serialization;
using Crm.Models;

public class Program
{
    public static int Main()
    {
        var cust = new Customer(
            Id: 42,
            Name: "Alice",
            Email: "alice@example.com",
            Tag: new List<string> { "vip", "retail" },
            Status: OrderStatus.Pending
        );

        if (!cust.Status.IsValid())
        {
            Console.WriteLine("OrderStatus should be valid");
            return 1;
        }

        if (cust.Status.ToXmlValue() != "pending")
        {
            Console.WriteLine("ToXmlValue mismatch");
            return 1;
        }

        // Test XmlSerializer roundtrip
        var serializer = new XmlSerializer(typeof(Customer));
        using var sw = new StringWriter();
        serializer.Serialize(sw, cust);
        var xml = sw.ToString();

        using var sr = new StringReader(xml);
        var decoded = (Customer?)serializer.Deserialize(sr);
        if (decoded == null)
        {
            Console.WriteLine("Deserialization produced null");
            return 1;
        }

        if (decoded.Id != 42 || decoded.Name != "Alice" || decoded.Email != "alice@example.com" || decoded.Status != OrderStatus.Pending)
        {
            Console.WriteLine("Field mismatch in deserialized object");
            return 1;
        }

        if (decoded.Tag == null || decoded.Tag.Count != 2 || decoded.Tag[0] != "vip")
        {
            Console.WriteLine("Tag list mismatch");
            return 1;
        }

        // Test System.Text.Json roundtrip
        var json = JsonSerializer.Serialize(cust);
        if (!json.Contains("\"id\":42") || !json.Contains("\"name\":\"Alice\"") || !json.Contains("\"Pending\""))
        {
            Console.WriteLine("JSON serialization missing fields: " + json);
            return 1;
        }

        var jsonDecoded = JsonSerializer.Deserialize<Customer>(json);
        if (jsonDecoded == null || jsonDecoded.Id != 42 || jsonDecoded.Name != "Alice" || jsonDecoded.Email != "alice@example.com" || jsonDecoded.Status != OrderStatus.Pending)
        {
            Console.WriteLine("JSON deserialization mismatch");
            return 1;
        }

        // Test IValidatableObject validation
        var validResults = new List<ValidationResult>();
        bool isValid = Validator.TryValidateObject(cust, new ValidationContext(cust), validResults, true);
        if (!isValid)
        {
            Console.WriteLine("Customer should be valid");
            return 1;
        }

        var invalidCust = new Customer(1, "Al", null, null, OrderStatus.Completed);
        var invalidResults = new List<ValidationResult>();
        bool isInvalid = !Validator.TryValidateObject(invalidCust, new ValidationContext(invalidCust), invalidResults, true);
        if (!isInvalid || invalidResults.Count == 0)
        {
            Console.WriteLine("Customer with short name should fail validation");
            return 1;
        }

        Console.WriteLine("Customer C# tests passed cleanly!");
        return 0;
    }
}
"#,
    )
    .unwrap();

    let build_status = dotnet_command()
        .args(["build", "--warnaserror"])
        .current_dir(temp.path())
        .status()
        .expect("Failed to run dotnet build");
    assert!(
        build_status.success(),
        "dotnet build failed on generated records"
    );

    let run_status = dotnet_command()
        .args(["run"])
        .current_dir(temp.path())
        .status()
        .expect("Failed to run dotnet run");
    assert!(
        run_status.success(),
        "dotnet run failed on generated records"
    );
}

#[test]
fn test_csharp_choice_polymorphic_hierarchy() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/payments");

    // Choice union: ContactChoice
    let choice_qname = QName::new(Some("https://example.com/payments"), "ContactChoice");
    ir.add_type(TypeDef::Union(UnionDef {
        qname: choice_qname.clone(),
        branches: vec![
            UnionBranch {
                variant_name: "email".into(),
                xml_name: "email".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: None,
            },
            UnionBranch {
                variant_name: "phone".into(),
                xml_name: "phone".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: None,
            },
        ],
        documentation: Some("Preferred contact channel".into()),
    }));

    // Struct: PaymentContact
    let payment_qname = QName::new(Some("https://example.com/payments"), "PaymentContact");
    ir.add_type(TypeDef::Struct(StructDef {
        qname: payment_qname.clone(),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef {
                name: "payer".into(),
                xml_name: "payer".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "contact".into(),
                xml_name: "contact".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Named(choice_qname),
                cardinality: Cardinality::optional_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
        ],
        documentation: None,
    }));

    let options = CSharpOptions {
        namespace: "Payments".to_string(),
        emit_xml_attributes: true,
        emit_json_attributes: true,
        emit_validation: true,
        record_kind: CSharpRecordKind::Class,
        use_file_scoped_namespaces: true,
        emit_root_records: true,
        ..Default::default()
    };

    let codegen = CSharpCodegen::new(options);
    let cs_code = codegen.generate_module(&ir);

    assert!(cs_code.contains("public abstract record ContactChoice"));
    assert!(cs_code.contains("[XmlInclude(typeof(ContactChoice.Email))]"));
    assert!(cs_code.contains("[XmlInclude(typeof(ContactChoice.Phone))]"));
    assert!(cs_code.contains("public sealed record Email("));
    assert!(cs_code.contains("public sealed record Phone("));
    assert!(cs_code.contains("XmlElement(\"email\", typeof(ContactChoice.Email))"));
    assert!(cs_code.contains("XmlElement(\"phone\", typeof(ContactChoice.Phone))"));
    assert!(cs_code.contains("JsonPropertyName(\"contact\")"));

    let temp = tempdir().unwrap();
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>"#;
    fs::write(temp.path().join("PaymentsApp.csproj"), csproj).unwrap();

    fs::write(temp.path().join("Models.cs"), &cs_code).unwrap();

    fs::write(
        temp.path().join("Program.cs"),
        r#"using System;
using System.IO;
using System.Xml.Serialization;
using Payments;

public class Program
{
    public static int Main()
    {
        var p1 = new PaymentContact(
            Payer: "Acme Corp",
            Contact: new ContactChoice.Email("billing@acme.com")
        );

        string channel = p1.Contact switch
        {
            ContactChoice.Email e => $"email:{e.Value}",
            ContactChoice.Phone p => $"phone:{p.Value}",
            _ => "unknown"
        };

        if (channel != "email:billing@acme.com")
        {
            Console.WriteLine($"Unexpected pattern match channel: {channel}");
            return 1;
        }

        // Test XmlSerializer roundtrip
        var serializer = new XmlSerializer(typeof(PaymentContact));
        using var sw = new StringWriter();
        serializer.Serialize(sw, p1);
        var xml = sw.ToString();

        using var sr = new StringReader(xml);
        var decoded = (PaymentContact?)serializer.Deserialize(sr);
        if (decoded == null || decoded.Payer != "Acme Corp")
        {
            Console.WriteLine("Deserialization failed");
            return 1;
        }

        if (decoded.Contact is not ContactChoice.Email emailChoice || emailChoice.Value != "billing@acme.com")
        {
            Console.WriteLine("Polymorphic choice deserialization failed");
            return 1;
        }

        Console.WriteLine("Choice pattern matching and XML serialization passed!");
        return 0;
    }
}
"#,
    )
    .unwrap();

    let build_status = dotnet_command()
        .args(["build", "--warnaserror"])
        .current_dir(temp.path())
        .status()
        .expect("Failed to run dotnet build");
    assert!(
        build_status.success(),
        "dotnet build failed on choice models"
    );

    let run_status = dotnet_command()
        .args(["run"])
        .current_dir(temp.path())
        .status()
        .expect("Failed to run dotnet run");
    assert!(run_status.success(), "dotnet run failed on choice models");
}

#[test]
fn test_csharp_recursive_cycle() {
    let mut ir = SchemaIR::new();
    let tree_qname = QName::new(None::<String>, "TreeNode");

    ir.add_type(TypeDef::Struct(StructDef {
        qname: tree_qname.clone(),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef {
                name: "label".into(),
                xml_name: "label".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "next".into(),
                xml_name: "next".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Named(tree_qname),
                cardinality: Cardinality::optional_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: true,
            },
        ],
        documentation: None,
    }));

    let options = CSharpOptions {
        namespace: "Tree".to_string(),
        emit_xml_attributes: true,
        emit_json_attributes: true,
        emit_validation: true,
        record_kind: CSharpRecordKind::Class,
        use_file_scoped_namespaces: true,
        emit_root_records: true,
        ..Default::default()
    };

    let codegen = CSharpCodegen::new(options);
    let cs_code = codegen.generate_module(&ir);

    assert!(cs_code.contains("public record TreeNode("));
    assert!(cs_code
        .contains("[property: XmlElement(\"label\"), JsonPropertyName(\"label\")] string Label,"));
    assert!(cs_code.contains(
        "[property: XmlElement(\"next\"), JsonPropertyName(\"next\")] TreeNode? Next = null"
    ));

    let temp = tempdir().unwrap();
    let csproj = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>"#;
    fs::write(temp.path().join("TreeApp.csproj"), csproj).unwrap();

    fs::write(temp.path().join("Models.cs"), &cs_code).unwrap();

    fs::write(
        temp.path().join("Program.cs"),
        r#"using System;
using System.IO;
using System.Xml.Serialization;
using Tree;

public class Program
{
    public static int Main()
    {
        var root = new TreeNode(
            Label: "root",
            Next: new TreeNode(
                Label: "child1",
                Next: new TreeNode(
                    Label: "child2"
                )
            )
        );

        var serializer = new XmlSerializer(typeof(TreeNode));
        using var sw = new StringWriter();
        serializer.Serialize(sw, root);
        var xml = sw.ToString();

        using var sr = new StringReader(xml);
        var decoded = (TreeNode?)serializer.Deserialize(sr);
        if (decoded == null || decoded.Label != "root")
        {
            Console.WriteLine("Root node mismatch");
            return 1;
        }

        if (decoded.Next == null || decoded.Next.Label != "child1" || decoded.Next.Next?.Label != "child2")
        {
            Console.WriteLine("Recursive child node mismatch");
            return 1;
        }

        Console.WriteLine("Recursive tree C# tests passed cleanly!");
        return 0;
    }
}
"#,
    )
    .unwrap();

    let build_status = dotnet_command()
        .args(["build", "--warnaserror"])
        .current_dir(temp.path())
        .status()
        .expect("Failed to run dotnet build");
    assert!(
        build_status.success(),
        "dotnet build failed on recursive tree models"
    );

    let run_status = dotnet_command()
        .args(["run"])
        .current_dir(temp.path())
        .status()
        .expect("Failed to run dotnet run");
    assert!(
        run_status.success(),
        "dotnet run failed on recursive tree models"
    );
}

#[test]
fn test_csharp_record_kind_from_str_loose() {
    assert_eq!(
        CSharpRecordKind::from_str_loose("class"),
        Some(CSharpRecordKind::Class)
    );
    assert_eq!(
        CSharpRecordKind::from_str_loose("record"),
        Some(CSharpRecordKind::Class)
    );
    assert_eq!(
        CSharpRecordKind::from_str_loose("struct"),
        Some(CSharpRecordKind::Struct)
    );
    assert_eq!(
        CSharpRecordKind::from_str_loose("record-struct"),
        Some(CSharpRecordKind::Struct)
    );
    assert_eq!(CSharpRecordKind::from_str_loose("invalid"), None);
}

#[test]
fn test_csharp_source_gen_context() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/crm");

    // Enum
    ir.add_type(TypeDef::Enum(EnumDef {
        qname: QName::new(Some("https://example.com/crm"), "OrderStatus"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        variants: vec![
            EnumValue {
                name: "pending".into(),
                value: "pending".into(),
                documentation: None,
            },
            EnumValue {
                name: "shipped".into(),
                value: "shipped".into(),
                documentation: None,
            },
        ],
        documentation: None,
    }));

    // Struct
    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/crm"), "Customer"),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef {
                name: "id".into(),
                xml_name: "id".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::Int),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "name".into(),
                xml_name: "name".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
        ],
        documentation: None,
    }));

    let options = CSharpOptions {
        namespace: "Crm.Models".to_string(),
        emit_source_gen: true,
        source_gen_context_name: "CrmJsonContext".to_string(),
        record_kind: CSharpRecordKind::Struct,
        ..Default::default()
    };

    let codegen = CSharpCodegen::new(options);
    let code = codegen.generate_module(&ir);

    // Record struct
    assert!(code.contains("public readonly record struct Customer("));

    // Source generation context
    assert!(code.contains("[JsonSourceGenerationOptions(WriteIndented = true)]"));
    assert!(code.contains("[JsonSerializable(typeof(Customer))]"));
    assert!(code.contains("[JsonSerializable(typeof(List<Customer>))]"));
    assert!(code.contains("[JsonSerializable(typeof(OrderStatus))]"));
    assert!(code.contains("public partial class CrmJsonContext : JsonSerializerContext"));
}
