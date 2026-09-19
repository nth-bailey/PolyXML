use polyxml::codegen::rust::{
    to_rust_field_identifier, to_rust_variant_identifier, RustCodegen, RustOptions,
};
use polyxml::ir::{
    Cardinality, EnumDef, EnumValue, FieldDef, FieldKind, PrimitiveType, QName, SchemaIR,
    StructDef, TypeDef, TypeRef, UnionBranch, UnionDef,
};

#[test]
fn test_rust_identifier_sanitization() {
    assert_eq!(to_rust_variant_identifier("pending"), "Pending");
    assert_eq!(to_rust_variant_identifier("in-progress"), "InProgress");
    assert_eq!(to_rust_variant_identifier("10-day-hold"), "Value10DayHold");
    assert_eq!(to_rust_variant_identifier(""), "Empty");

    assert_eq!(to_rust_field_identifier("type"), "r#type");
    assert_eq!(to_rust_field_identifier("match"), "r#match");
    assert_eq!(to_rust_field_identifier("box"), "r#box");
    assert_eq!(to_rust_field_identifier("100mDash"), "_100m_dash");
    assert_eq!(to_rust_field_identifier("normalField"), "normal_field");
}

#[test]
fn test_rust_zero_copy_codegen() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/crm");

    // Dimension: pure numeric, should NOT have lifetime <'a>
    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/crm"), "Dimension"),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef::new(
                "width",
                "width",
                FieldKind::Element,
                TypeRef::Primitive(PrimitiveType::Double),
            ),
            FieldDef::new(
                "height",
                "height",
                FieldKind::Element,
                TypeRef::Primitive(PrimitiveType::Double),
            ),
        ],
        documentation: Some("Fixed size dimensions".into()),
    }));

    // Customer: has strings, MUST have lifetime <'a>
    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/crm"), "Customer"),
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
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "type".into(), // keyword
                xml_name: "type".into(),
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
                name: "size".into(),
                xml_name: "size".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Named(QName::new(Some("https://example.com/crm"), "Dimension")),
                cardinality: Cardinality::optional_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false,
            },
            FieldDef {
                name: "notes".into(),
                xml_name: "note".into(),
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
        documentation: Some("Customer record".into()),
    }));

    let codegen = RustCodegen::new(RustOptions {
        zero_copy: true,
        derive_serde: true,
        derive_default: true,
        emit_polyxml_attrs: true,
        emit_root_aliases: true,
    });

    let code = codegen.generate_module(&ir);

    assert!(code.contains("use std::borrow::Cow;"));
    assert!(code.contains("use serde::{Deserialize, Serialize};"));

    // Dimension has no lifetimes
    assert!(code.contains("pub struct Dimension {"));
    assert!(code.contains("pub width: f64,"));
    assert!(code.contains("pub height: f64,"));

    // Customer has lifetime 'a
    assert!(code.contains("pub struct Customer<'a> {"));
    assert!(code.contains("pub id: i32,"));
    assert!(code.contains("pub name: Cow<'a, str>,"));
    assert!(code.contains("pub r#type: Option<Cow<'a, str>>,"));
    assert!(code.contains("pub size: Option<Dimension>,"));
    assert!(code.contains("pub notes: Vec<Cow<'a, str>>,"));

    // Field attributes
    assert!(code.contains("#[polyxml(attribute = \"id\")]"));
    assert!(code.contains("#[polyxml(element = \"name\")]"));
    assert!(code.contains("#[polyxml(element = \"type\")]"));
}

#[test]
fn test_rust_owned_codegen() {
    let mut ir = SchemaIR::new();

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::local("User"),
        base_type: None,
        is_abstract: false,
        fields: vec![FieldDef::new(
            "username",
            "username",
            FieldKind::Element,
            TypeRef::Primitive(PrimitiveType::String),
        )],
        documentation: None,
    }));

    let codegen = RustCodegen::new(RustOptions {
        zero_copy: false, // Owned mode
        derive_serde: true,
        derive_default: true,
        emit_polyxml_attrs: true,
        emit_root_aliases: true,
    });

    let code = codegen.generate_module(&ir);

    // No std::borrow::Cow imported or used
    assert!(!code.contains("use std::borrow::Cow;"));
    assert!(code.contains("pub struct User {"));
    assert!(code.contains("pub username: String,"));
}

#[test]
fn test_rust_enum_and_methods() {
    let mut ir = SchemaIR::new();

    ir.add_type(TypeDef::Enum(EnumDef {
        qname: QName::local("DeliveryStatus"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        variants: vec![
            EnumValue {
                name: "pending".into(),
                value: "pending".into(),
                documentation: Some("Awaiting pickup".into()),
            },
            EnumValue {
                name: "in-transit".into(),
                value: "in-transit".into(),
                documentation: None,
            },
            EnumValue {
                name: "delivered".into(),
                value: "delivered".into(),
                documentation: None,
            },
        ],
        documentation: Some("Parcel state".into()),
    }));

    let codegen = RustCodegen::new(RustOptions::default());
    let code = codegen.generate_module(&ir);

    assert!(code.contains("pub enum DeliveryStatus {"));
    assert!(code.contains("Pending,"));
    assert!(code.contains("InTransit,"));
    assert!(code.contains("Delivered,"));

    // Methods
    assert!(code.contains("impl DeliveryStatus {"));
    assert!(code.contains("pub fn as_str(&self) -> &'static str"));
    assert!(code.contains("Self::InTransit => \"in-transit\""));

    // FromStr
    assert!(code.contains("impl std::str::FromStr for DeliveryStatus"));
    assert!(code.contains("\"in-transit\" => Ok(Self::InTransit)"));

    // Display
    assert!(code.contains("impl std::fmt::Display for DeliveryStatus"));
}

#[test]
fn test_rust_choice_union_codegen() {
    let mut ir = SchemaIR::new();

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::local("Card"),
        base_type: None,
        is_abstract: false,
        fields: vec![FieldDef::new(
            "number",
            "number",
            FieldKind::Element,
            TypeRef::Primitive(PrimitiveType::String),
        )],
        documentation: None,
    }));

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::local("Cash"),
        base_type: None,
        is_abstract: false,
        fields: vec![FieldDef::new(
            "amount",
            "amount",
            FieldKind::Element,
            TypeRef::Primitive(PrimitiveType::Decimal),
        )],
        documentation: None,
    }));

    ir.add_type(TypeDef::Union(UnionDef {
        qname: QName::local("PaymentMethod"),
        branches: vec![
            UnionBranch {
                variant_name: "Card".into(),
                xml_name: "card".into(),
                namespace: None,
                type_ref: TypeRef::Named(QName::local("Card")),
                documentation: None,
            },
            UnionBranch {
                variant_name: "Cash".into(),
                xml_name: "cash".into(),
                namespace: None,
                type_ref: TypeRef::Named(QName::local("Cash")),
                documentation: None,
            },
        ],
        documentation: Some("Payment union".into()),
    }));

    let codegen = RustCodegen::new(RustOptions::default());
    let code = codegen.generate_module(&ir);

    // PaymentMethod references Card which has String -> needs 'a
    assert!(code.contains("pub enum PaymentMethod<'a> {"));
    assert!(code.contains("Card(Card<'a>),"));
    assert!(code.contains("Cash(Cash),"));
}

#[test]
fn test_rust_recursive_cycle_boxing() {
    let mut ir = SchemaIR::new();

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::local("TreeNode"),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef::new(
                "value",
                "value",
                FieldKind::Element,
                TypeRef::Primitive(PrimitiveType::String),
            ),
            FieldDef {
                name: "left".into(),
                xml_name: "left".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Named(QName::local("TreeNode")),
                cardinality: Cardinality::optional_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: true, // Tarjan cycle cut point
            },
            FieldDef {
                name: "right".into(),
                xml_name: "right".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::Named(QName::local("TreeNode")),
                cardinality: Cardinality::optional_one(),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: true, // Tarjan cycle cut point
            },
            FieldDef {
                name: "children".into(),
                xml_name: "child".into(),
                namespace: None,
                kind: FieldKind::Element,
                type_ref: TypeRef::List(Box::new(TypeRef::Named(QName::local("TreeNode")))),
                cardinality: Cardinality::unbounded(0),
                nillable: false,
                default_value: None,
                fixed_value: None,
                documentation: None,
                facets: None,
                is_cycle_cut: false, // Vec does not require Box
            },
        ],
        documentation: None,
    }));

    let codegen = RustCodegen::new(RustOptions::default());
    let code = codegen.generate_module(&ir);

    assert!(code.contains("pub struct TreeNode<'a> {"));
    assert!(code.contains("pub value: Cow<'a, str>,"));
    assert!(code.contains("pub left: Option<Box<TreeNode<'a>>>,"));
    assert!(code.contains("pub right: Option<Box<TreeNode<'a>>>,"));
    assert!(code.contains("pub children: Vec<TreeNode<'a>>,"));
}
