use std::fs;
use std::process::Command;
use tempfile::tempdir;

use polyxml::codegen::java::{
    to_java_enum_constant, to_java_field_identifier, to_java_type_name, JavaBackend, JavaCodegen,
    JavaOptions,
};
use polyxml::ir::{
    Cardinality, EnumDef, EnumValue, FieldDef, FieldKind, PrimitiveType, QName, RestrictionFacets,
    SchemaIR, SimpleTypeDef, StructDef, TypeDef, TypeRef, UnionBranch, UnionDef,
};

#[test]
fn test_java_identifier_sanitization() {
    assert_eq!(to_java_field_identifier("class"), "class_");
    assert_eq!(to_java_field_identifier("record"), "record_");
    assert_eq!(to_java_field_identifier("sealed"), "sealed_");
    assert_eq!(to_java_field_identifier("permits"), "permits_");
    assert_eq!(to_java_field_identifier("import"), "import_");
    assert_eq!(to_java_field_identifier("default"), "default_");
    assert_eq!(to_java_field_identifier("normalField"), "normalField");
    assert_eq!(
        to_java_field_identifier("snake_case_field"),
        "snakeCaseField"
    );
    assert_eq!(to_java_field_identifier("100mDash"), "_100mDash");

    assert_eq!(to_java_type_name("order-status"), "OrderStatus");
    assert_eq!(to_java_type_name("customer_record"), "CustomerRecord");
    assert_eq!(to_java_type_name("100Percent"), "Type100percent");

    assert_eq!(to_java_enum_constant("pending"), "PENDING");
    assert_eq!(to_java_enum_constant("in-progress"), "IN_PROGRESS");
    assert_eq!(to_java_enum_constant("10-day-hold"), "VALUE_10_DAY_HOLD");
    assert_eq!(to_java_enum_constant(""), "EMPTY");
}

#[test]
fn test_java_records_and_enums_generation() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/shop");

    // Enum: OrderStatus
    ir.add_type(TypeDef::Enum(EnumDef {
        qname: QName::new(Some("https://example.com/shop"), "OrderStatus"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        variants: vec![
            EnumValue {
                name: "pending".into(),
                value: "pending".into(),
                documentation: Some("Pending review".into()),
            },
            EnumValue {
                name: "shipped".into(),
                value: "shipped".into(),
                documentation: None,
            },
            EnumValue {
                name: "cancelled".into(),
                value: "cancelled".into(),
                documentation: None,
            },
        ],
        documentation: Some("Status of order processing".into()),
    }));

    // Struct: Customer
    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/shop"), "Customer"),
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
                documentation: Some("Customer unique ID".into()),
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
                facets: Some(RestrictionFacets {
                    min_length: Some(2),
                    max_length: Some(50),
                    patterns: vec!["^[A-Za-z ]+$".into()],
                    ..Default::default()
                }),
                is_cycle_cut: false,
            },
            FieldDef {
                name: "class".into(), // keyword in Java
                xml_name: "class".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::optional_one(),
                nillable: true,
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
                type_ref: TypeRef::Named(QName::new(
                    Some("https://example.com/shop"),
                    "OrderStatus",
                )),
                cardinality: Cardinality::required_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "tags".into(),
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
        ],
        documentation: Some("Customer record with orders".into()),
    }));

    let options = JavaOptions {
        package_name: "com.example.shop".to_string(),
        backend: JavaBackend::Standard,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let files = codegen.generate_files(&ir);

    assert_eq!(files.len(), 2);
    let files_map: std::collections::HashMap<_, _> = files.into_iter().collect();

    // Verify OrderStatus.java
    let order_status_code = files_map.get("OrderStatus.java").unwrap();
    assert!(order_status_code.contains("package com.example.shop;"));
    assert!(order_status_code.contains("public enum OrderStatus {"));
    assert!(order_status_code.contains("PENDING(\"pending\"),"));
    assert!(order_status_code.contains("SHIPPED(\"shipped\"),"));
    assert!(order_status_code.contains("CANCELLED(\"cancelled\");"));
    assert!(order_status_code.contains("public String getValue()"));
    assert!(order_status_code.contains("public static OrderStatus fromValue(String value)"));

    // Verify Customer.java
    let customer_code = files_map.get("Customer.java").unwrap();
    assert!(customer_code.contains("package com.example.shop;"));
    assert!(customer_code.contains("public record Customer("));
    assert!(customer_code.contains("int id,"));
    assert!(customer_code.contains("String name,"));
    assert!(customer_code.contains("java.util.Optional<String> class_,"));
    assert!(customer_code.contains("OrderStatus status,"));
    assert!(customer_code.contains("java.util.List<String> tags"));
    assert!(customer_code.contains("Objects.requireNonNull(name, \"name must not be null\");"));
    assert!(customer_code.contains(
        "if (name.length() < 2) throw new IllegalArgumentException(\"name minLength is 2\");"
    ));
    assert!(customer_code.contains(
        "if (name.length() > 50) throw new IllegalArgumentException(\"name maxLength is 50\");"
    ));

    // Verify compilation with javac -Werror
    let dir = tempdir().unwrap();
    for (filename, content) in &files_map {
        fs::write(dir.path().join(filename), content).unwrap();
    }

    let javac = Command::new("javac")
        .arg("-Werror")
        .arg(dir.path().join("OrderStatus.java"))
        .arg(dir.path().join("Customer.java"))
        .output();

    if let Ok(out) = javac {
        assert!(
            out.status.success(),
            "javac failed on generated records: {}\nstdout: {}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

#[test]
fn test_java_sealed_interface_choice() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/payment");

    ir.add_type(TypeDef::Union(UnionDef {
        qname: QName::new(Some("https://example.com/payment"), "PaymentChoice"),
        branches: vec![
            UnionBranch {
                variant_name: "creditCard".into(),
                xml_name: "creditCard".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: Some("Credit card token".into()),
            },
            UnionBranch {
                variant_name: "directDebit".into(),
                xml_name: "directDebit".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: Some("IBAN direct debit".into()),
            },
        ],
        documentation: Some("Choice of payment method".into()),
    }));

    let options = JavaOptions {
        package_name: "com.example.payment".to_string(),
        backend: JavaBackend::Standard,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let files = codegen.generate_files(&ir);

    assert_eq!(files.len(), 1);
    let (filename, content) = &files[0];
    assert_eq!(filename, "PaymentChoice.java");
    assert!(content.contains("package com.example.payment;"));
    assert!(content.contains("public sealed interface PaymentChoice permits PaymentChoice.CreditCard, PaymentChoice.DirectDebit {"));
    assert!(content.contains("record CreditCard(String value) implements PaymentChoice {}"));
    assert!(content.contains("record DirectDebit(String value) implements PaymentChoice {}"));

    // Verify compilation with javac -Werror
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("PaymentChoice.java"), content).unwrap();

    let javac = Command::new("javac")
        .arg("-Werror")
        .arg(dir.path().join("PaymentChoice.java"))
        .output();

    if let Ok(out) = javac {
        assert!(
            out.status.success(),
            "javac failed on sealed interface: {}\nstdout: {}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

#[test]
fn test_java_module_container_class() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/banking");

    ir.add_type(TypeDef::Simple(Box::new(SimpleTypeDef {
        qname: QName::new(Some("https://example.com/banking"), "Iban"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        facets: RestrictionFacets {
            min_length: Some(15),
            max_length: Some(34),
            patterns: vec!["^[A-Z]{2}[0-9]{2}[A-Z0-9]+$".into()],
            ..Default::default()
        },
        documentation: Some("International Bank Account Number".into()),
    })));

    let options = JavaOptions {
        package_name: "com.example.banking".to_string(),
        backend: JavaBackend::Standard,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let code = codegen.generate_module(&ir, "BankingModels");

    assert!(code.contains("package com.example.banking;"));
    assert!(code.contains("public final class BankingModels {"));
    assert!(code.contains("private BankingModels() {}"));
    assert!(code.contains("public static record Iban(String value) {"));
    assert!(code.contains(
        "if (value.length() < 15) throw new IllegalArgumentException(\"value minLength is 15\");"
    ));
    assert!(code.contains(
        "if (value.length() > 34) throw new IllegalArgumentException(\"value maxLength is 34\");"
    ));

    // Verify compilation with javac -Werror
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("BankingModels.java"), code).unwrap();

    let javac = Command::new("javac")
        .arg("-Werror")
        .arg(dir.path().join("BankingModels.java"))
        .output();

    if let Ok(out) = javac {
        assert!(
            out.status.success(),
            "javac failed on BankingModels.java: {}\nstdout: {}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

#[test]
fn test_java_jackson_backend_struct_annotations() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/crm");

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/crm"), "Contact"),
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
                name: "email".into(),
                xml_name: "email".into(),
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
                name: "phone".into(),
                xml_name: "phone".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                cardinality: Cardinality::optional_one(),
                nillable: true,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "tags".into(),
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
        ],
        documentation: Some("CRM contact record".into()),
    }));

    let options = JavaOptions {
        package_name: "com.example.crm".to_string(),
        backend: JavaBackend::Jackson,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let files = codegen.generate_files(&ir);

    assert_eq!(files.len(), 1);
    let (filename, code) = &files[0];
    assert_eq!(filename, "Contact.java");

    // Jackson imports
    assert!(code.contains("import com.fasterxml.jackson.annotation.*;"));
    assert!(code.contains("import com.fasterxml.jackson.dataformat.xml.annotation.*;"));

    // Class-level annotations
    assert!(code.contains("@JsonIgnoreProperties(ignoreUnknown = true)"));
    assert!(code.contains("@JsonInclude(JsonInclude.Include.NON_EMPTY)"));
    assert!(code.contains("@JacksonXmlRootElement(localName = \"Contact\""));

    // Field annotations — attribute
    assert!(code.contains("@JsonProperty(\"id\")"));
    assert!(code.contains("@JacksonXmlProperty(localName = \"id\", isAttribute = true)"));

    // Field annotations — element
    assert!(code.contains("@JsonProperty(\"email\")"));
    assert!(code.contains("@JacksonXmlProperty(localName = \"email\", isAttribute = false)"));

    // Optional field gets NON_EMPTY
    assert!(code.contains("@JsonProperty(\"phone\")"));

    // List field gets wrapper suppression
    assert!(code.contains("@JacksonXmlElementWrapper(useWrapping = false)"));
}

#[test]
fn test_java_jackson_backend_enum_annotations() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/orders");

    ir.add_type(TypeDef::Enum(EnumDef {
        qname: QName::new(Some("https://example.com/orders"), "Priority"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        variants: vec![
            EnumValue {
                name: "low".into(),
                value: "low".into(),
                documentation: None,
            },
            EnumValue {
                name: "high".into(),
                value: "high".into(),
                documentation: None,
            },
        ],
        documentation: None,
    }));

    let options = JavaOptions {
        package_name: "com.example.orders".to_string(),
        backend: JavaBackend::Jackson,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let files = codegen.generate_files(&ir);

    assert_eq!(files.len(), 1);
    let (_, code) = &files[0];

    // @JsonValue on getValue()
    assert!(code.contains("@JsonValue"));
    assert!(code.contains("public String getValue()"));

    // @JsonCreator on fromValue()
    assert!(code.contains("@JsonCreator"));
    assert!(code.contains("public static Priority fromValue(String value)"));
}

#[test]
fn test_java_jackson_backend_union_annotations() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/messaging");

    ir.add_type(TypeDef::Union(UnionDef {
        qname: QName::new(Some("https://example.com/messaging"), "MessageChannel"),
        branches: vec![
            UnionBranch {
                variant_name: "sms".into(),
                xml_name: "sms".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: None,
            },
            UnionBranch {
                variant_name: "email".into(),
                xml_name: "email".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: None,
            },
        ],
        documentation: None,
    }));

    let options = JavaOptions {
        package_name: "com.example.messaging".to_string(),
        backend: JavaBackend::Jackson,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let files = codegen.generate_files(&ir);

    assert_eq!(files.len(), 1);
    let (_, code) = &files[0];

    // Polymorphic type info
    assert!(code.contains("@JsonTypeInfo(use = JsonTypeInfo.Id.NAME"));
    assert!(code.contains("@JsonSubTypes({"));
    assert!(code.contains("@JsonSubTypes.Type(value = MessageChannel.Sms.class, name = \"sms\")"));
    assert!(
        code.contains("@JsonSubTypes.Type(value = MessageChannel.Email.class, name = \"email\")")
    );

    // Variant type names
    assert!(code.contains("@JsonTypeName(\"sms\")"));
    assert!(code.contains("@JsonTypeName(\"email\")"));
}

#[test]
fn test_java_jackson_backend_simple_type_annotations() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/types");

    ir.add_type(TypeDef::Simple(Box::new(SimpleTypeDef {
        qname: QName::new(Some("https://example.com/types"), "CurrencyCode"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        facets: RestrictionFacets {
            length: Some(3),
            patterns: vec!["^[A-Z]{3}$".into()],
            ..Default::default()
        },
        documentation: Some("ISO 4217 currency code".into()),
    })));

    let options = JavaOptions {
        package_name: "com.example.types".to_string(),
        backend: JavaBackend::Jackson,
        use_records: true,
        validate_facets: true,
        emit_root_aliases: true,
        custom_header: None,
    };
    let codegen = JavaCodegen::new(options);
    let files = codegen.generate_files(&ir);

    assert_eq!(files.len(), 1);
    let (_, code) = &files[0];

    // @JsonValue @JacksonXmlText on value component
    assert!(code.contains("@JsonValue @JacksonXmlText String value"));

    // @JsonCreator factory method
    assert!(code.contains("@JsonCreator"));
    assert!(code.contains("public static CurrencyCode of(String value)"));
    assert!(code.contains("return new CurrencyCode(value);"));

    // Validation still present
    assert!(code.contains("if (value.length() != 3)"));
}

#[test]
fn test_java_jackson_backend_from_str_loose() {
    assert_eq!(
        JavaBackend::from_str_loose("jackson"),
        Some(JavaBackend::Jackson)
    );
    assert_eq!(
        JavaBackend::from_str_loose("Jackson"),
        Some(JavaBackend::Jackson)
    );
    assert_eq!(
        JavaBackend::from_str_loose("spring"),
        Some(JavaBackend::Jackson)
    );
    assert_eq!(
        JavaBackend::from_str_loose("spring-boot"),
        Some(JavaBackend::Jackson)
    );
    assert_eq!(
        JavaBackend::from_str_loose("enterprise"),
        Some(JavaBackend::Jackson)
    );
    assert_eq!(
        JavaBackend::from_str_loose("standard"),
        Some(JavaBackend::Standard)
    );
    assert_eq!(
        JavaBackend::from_str_loose("std"),
        Some(JavaBackend::Standard)
    );
    assert_eq!(JavaBackend::from_str_loose("unknown"), None);
}
