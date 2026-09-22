use polyxml::codegen::rust::{
    to_rust_field_identifier, to_rust_variant_identifier, RustCodegen, RustOptions,
};
use polyxml::ir::{
    Cardinality, EnumDef, EnumValue, FieldDef, FieldKind, PrimitiveType, QName, SchemaIR,
    StructDef, TypeDef, TypeRef, UnionBranch, UnionDef,
};
use polyxml::schema_parser::XsdParser;

/// Slice a generated struct or `impl` body from `start` up to the first
/// closing brace at column 0, panicking when `start` is absent. Scoping
/// assertions to one body keeps them from passing against a sibling type.
fn section<'a>(code: &'a str, start: &str) -> &'a str {
    let from = code
        .find(start)
        .unwrap_or_else(|| panic!("expected {start:?} in generated code:\n{code}"));
    let rest = &code[from..];
    match rest.find("\n}\n") {
        Some(i) => &rest[..i + 3],
        None => rest,
    }
}

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
        emit_codecs: false,
        emit_rkyv: false,
        custom_header: None,
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
        emit_codecs: false,
        emit_rkyv: false,
        custom_header: None,
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

#[test]
fn test_rust_codecs_codegen() {
    let mut ir = SchemaIR::new();

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::local("Order"),
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
                name: "customer".into(),
                xml_name: "customer".into(),
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

    let codegen_enabled = RustCodegen::new(RustOptions {
        emit_codecs: true,
        ..Default::default()
    });
    let code_enabled = codegen_enabled.generate_module(&ir);
    assert!(code_enabled.contains("pub fn from_xml(xml: &'a str) -> Result<Self>"));
    assert!(code_enabled.contains("pub fn from_xml_bytes(xml_bytes: &'a [u8]) -> Result<Self>"));
    assert!(code_enabled.contains(
        "pub fn from_json_str(json_str: &'a str) -> std::result::Result<Self, serde_json::Error>"
    ));
    assert!(code_enabled.contains(
        "pub fn from_json_slice(bytes: &'a [u8]) -> std::result::Result<Self, serde_json::Error>"
    ));
    assert!(code_enabled.contains(
        "pub fn decode_xml(reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self>"
    ));
    assert!(code_enabled.contains("pub fn to_xml(&self) -> Result<Vec<u8>>"));
    assert!(code_enabled.contains("pub fn to_xml_string(&self) -> Result<String>"));
    assert!(code_enabled.contains(
        "pub fn to_json_string(&self) -> std::result::Result<String, serde_json::Error>"
    ));
    assert!(code_enabled
        .contains("pub fn to_json_vec(&self) -> std::result::Result<Vec<u8>, serde_json::Error>"));
    assert!(code_enabled.contains("pub fn encode_xml<W: std::io::Write>(&self, writer: &mut Writer<W>, tag_name: Option<&str>) -> Result<()>"));

    let codegen_disabled = RustCodegen::new(RustOptions {
        emit_codecs: false,
        ..Default::default()
    });
    let code_disabled = codegen_disabled.generate_module(&ir);
    assert!(!code_disabled.contains("pub fn from_xml("));
    assert!(!code_disabled.contains("pub fn to_xml(&self)"));
    assert!(!code_disabled.contains("pub fn from_json_str("));
    assert!(!code_disabled.contains("pub fn to_json_string("));
}

#[test]
fn test_rust_attribute_codec_loop_syntax() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/siri");
    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/siri"), "Envelope"),
        base_type: None,
        is_abstract: false,
        fields: vec![
            FieldDef {
                name: "version".into(),
                xml_name: "version".into(),
                namespace: None,
                kind: FieldKind::Attribute,
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
                name: "body".into(),
                xml_name: "Body".into(),
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

    let codegen = RustCodegen::new(RustOptions {
        emit_codecs: true,
        ..Default::default()
    });
    let code = codegen.generate_module(&ir);
    assert!(code.contains("for attr in start.attributes() {"));
    assert!(code.contains("match attr.key.local_name().as_ref() {"));
    assert!(code.contains("\"version\" => {"));
}

#[test]
fn test_rust_rkyv_derives() {
    let mut ir = SchemaIR::new().with_target_namespace("https://example.com/rkyv");

    ir.add_type(TypeDef::Enum(EnumDef {
        qname: QName::new(Some("https://example.com/rkyv"), "DeviceStatus"),
        base_type: TypeRef::Primitive(PrimitiveType::String),
        variants: vec![
            EnumValue {
                name: "active".into(),
                value: "active".into(),
                documentation: None,
            },
            EnumValue {
                name: "idle".into(),
                value: "idle".into(),
                documentation: None,
            },
        ],
        documentation: None,
    }));

    ir.add_type(TypeDef::Union(UnionDef {
        qname: QName::new(Some("https://example.com/rkyv"), "Payload"),
        branches: vec![
            UnionBranch {
                variant_name: "text".into(),
                xml_name: "text".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::String),
                documentation: None,
            },
            UnionBranch {
                variant_name: "number".into(),
                xml_name: "number".into(),
                namespace: None,
                type_ref: TypeRef::Primitive(PrimitiveType::Int),
                documentation: None,
            },
        ],
        documentation: None,
    }));

    ir.add_type(TypeDef::Struct(StructDef {
        qname: QName::new(Some("https://example.com/rkyv"), "Packet"),
        base_type: None,
        is_abstract: false,
        fields: vec![FieldDef {
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
        }],
        documentation: None,
    }));

    let codegen = RustCodegen::new(RustOptions {
        emit_rkyv: true,
        ..Default::default()
    });
    let code = codegen.generate_module(&ir);

    // Verify rkyv derives on enum, union, struct
    assert!(code.contains("#[cfg_attr(feature = \"rkyv\", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]"));
    assert!(code.contains("#[cfg_attr(feature = \"rkyv\", rkyv(check_bytes))]"));
}

// ---------------------------------------------------------------------------
// xsd:extension base-field flattening: Rust structs have no inheritance, so a
// derived type must inline the base chain's fields everywhere — struct body,
// lifetime analysis, and both codec directions.
// ---------------------------------------------------------------------------

/// Lifetime edge case: `Middle` owns no string fields, so it only earns its
/// `<'a>` if inherited fields participate in `compute_types_with_lifetime`.
fn extension_xsd() -> &'static str {
    r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            targetNamespace="urn:ext" xmlns:t="urn:ext"
            elementFormDefault="qualified">
        <xs:complexType name="Base">
            <xs:sequence>
                <xs:element name="label" type="xs:string"/>
            </xs:sequence>
            <xs:attribute name="kind" type="xs:string" use="optional"/>
        </xs:complexType>
        <xs:complexType name="Middle">
            <xs:complexContent>
                <xs:extension base="t:Base">
                    <xs:sequence>
                        <xs:element name="rank" type="xs:int"/>
                    </xs:sequence>
                    <xs:attribute name="active" type="xs:boolean" use="optional"/>
                </xs:extension>
            </xs:complexContent>
        </xs:complexType>
        <xs:complexType name="Leaf">
            <xs:complexContent>
                <xs:extension base="t:Middle">
                    <xs:sequence>
                        <xs:element name="note" type="xs:string" minOccurs="0"/>
                    </xs:sequence>
                </xs:extension>
            </xs:complexContent>
        </xs:complexType>
    </xs:schema>"#
}

#[test]
fn test_rust_extension_inlines_base_fields() {
    let ir = XsdParser::new()
        .parse_str(extension_xsd())
        .expect("parse extension schema");
    let code = RustCodegen::new(RustOptions::default()).generate_module(&ir);

    let base = section(&code, "pub struct Base");
    assert!(base.contains("pub label: Cow<'a, str>"));
    assert!(base.contains("pub kind: Option<Cow<'a, str>>"));

    let middle = section(&code, "pub struct Middle");
    assert!(
        middle.contains("pub struct Middle<'a> {"),
        "inherited string field must propagate the lifetime parameter:\n{middle}"
    );
    for inherited in [
        "pub label: Cow<'a, str>,",
        "pub kind: Option<Cow<'a, str>>,",
    ] {
        assert!(
            middle.contains(inherited),
            "Middle dropped inherited field {inherited:?}:\n{middle}"
        );
    }
    assert!(middle.contains("pub rank: i32,"));
    assert!(middle.contains("pub active: Option<bool>,"));
    // Root-first ordering: base fields precede derived ones.
    let label_at = middle.find("pub label:").expect("label");
    let rank_at = middle.find("pub rank:").expect("rank");
    assert!(label_at < rank_at, "base fields must come first:\n{middle}");

    // Three-level chain: Leaf sees Base + Middle + its own field.
    let leaf = section(&code, "pub struct Leaf");
    for expected in [
        "pub label: Cow<'a, str>,",
        "pub kind: Option<Cow<'a, str>>,",
        "pub rank: i32,",
        "pub active: Option<bool>,",
        "pub note: Option<Cow<'a, str>>,",
    ] {
        assert!(
            leaf.contains(expected),
            "Leaf missing {expected:?}:\n{leaf}"
        );
    }
}

#[test]
fn test_rust_extension_codecs_decode_and_encode_base_fields() {
    let ir = XsdParser::new()
        .parse_str(extension_xsd())
        .expect("parse extension schema");
    let code = RustCodegen::new(RustOptions::default()).generate_module(&ir);

    let middle_impl = section(&code, "impl<'a> Middle");
    // Decode: variable slots, element dispatch, and attribute dispatch for
    // every inherited member, plus enforcement of required base fields.
    for expected in [
        "let mut var_label = None;",
        "let mut var_kind = None;",
        "let mut var_rank = None;",
        "\"label\" => {",
        "\"kind\" => {",
        "\"rank\" => {",
        "\"active\" => {",
        "Missing required field 'label'",
    ] {
        assert!(
            middle_impl.contains(expected),
            "Middle codec missing {expected:?}:\n{middle_impl}"
        );
    }
    // Encode: inherited element and attribute both written back out.
    for expected in [
        "BytesStart::new(\"label\")",
        "BytesText::new(self.label.as_ref())",
        "push_attribute((\"kind\"",
        "BytesStart::new(\"rank\")",
    ] {
        assert!(
            middle_impl.contains(expected),
            "Middle encode missing {expected:?}:\n{middle_impl}"
        );
    }

    let leaf_impl = section(&code, "impl<'a> Leaf");
    for expected in [
        "let mut var_label = None;",
        "let mut var_rank = None;",
        "\"label\" => {",
        "\"rank\" => {",
        "Missing required field 'label'",
        "BytesStart::new(\"label\")",
        "push_attribute((\"kind\"",
    ] {
        assert!(
            leaf_impl.contains(expected),
            "Leaf codec missing {expected:?}:\n{leaf_impl}"
        );
    }
}

#[test]
fn test_rust_simple_content_extension_shadow_rule() {
    // simpleContent extension re-declares the base `value` field; flattening
    // must inherit the base attributes without emitting a `value_2` duplicate.
    let xsd = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            targetNamespace="urn:sc" xmlns:t="urn:sc">
        <xs:complexType name="Measurement">
            <xs:simpleContent>
                <xs:extension base="xs:decimal">
                    <xs:attribute name="unit" type="xs:string"/>
                </xs:extension>
            </xs:simpleContent>
        </xs:complexType>
        <xs:complexType name="PreciseMeasurement">
            <xs:simpleContent>
                <xs:extension base="t:Measurement">
                    <xs:attribute name="precision" type="xs:int"/>
                </xs:extension>
            </xs:simpleContent>
        </xs:complexType>
    </xs:schema>"#;

    let ir = XsdParser::new()
        .parse_str(xsd)
        .expect("parse simpleContent schema");
    let code = RustCodegen::new(RustOptions::default()).generate_module(&ir);

    let derived = section(&code, "pub struct PreciseMeasurement");
    assert!(
        derived.contains("pub unit: Option<Cow<'a, str>>,"),
        "inherited unit attribute missing:\n{derived}"
    );
    assert!(
        derived.contains("pub precision: Option<i32>,"),
        "own precision attribute missing:\n{derived}"
    );
    assert_eq!(
        derived.matches("pub value").count(),
        1,
        "value must be shadowed by the derived declaration, not duplicated:\n{derived}"
    );
    assert!(
        !derived.contains("value_2"),
        "shadow rule must prevent suffixed duplicates:\n{derived}"
    );

    let derived_impl = section(&code, "impl<'a> PreciseMeasurement");
    assert!(derived_impl.contains("\"unit\" => {"));
    assert!(derived_impl.contains("\"precision\" => {"));
    assert!(derived_impl.contains("push_attribute((\"unit\""));
    assert!(derived_impl.contains("push_attribute((\"precision\""));
}

#[test]
fn test_rust_extension_base_cycle_cut() {
    // A circular extension is invalid XSD but must not hang or duplicate
    // fields when handed to the generator directly.
    let mut ir = SchemaIR::new().with_target_namespace("urn:cycle");
    let qa = QName::new(Some("urn:cycle"), "Alpha");
    let qb = QName::new(Some("urn:cycle"), "Beta");

    ir.add_type(TypeDef::Struct(StructDef {
        qname: qa.clone(),
        base_type: Some(qb.clone()),
        is_abstract: false,
        fields: vec![FieldDef::new(
            "alphaField",
            "alphaField",
            FieldKind::Element,
            TypeRef::Primitive(PrimitiveType::String),
        )],
        documentation: None,
    }));
    ir.add_type(TypeDef::Struct(StructDef {
        qname: qb.clone(),
        base_type: Some(qa.clone()),
        is_abstract: false,
        fields: vec![FieldDef::new(
            "betaField",
            "betaField",
            FieldKind::Element,
            TypeRef::Primitive(PrimitiveType::Int),
        )],
        documentation: None,
    }));

    let codegen = RustCodegen::new(RustOptions::default());
    let code = codegen.generate_module(&ir);

    let alpha = section(&code, "pub struct Alpha");
    assert_eq!(
        alpha.matches("pub beta_field").count(),
        1,
        "cycle-merged field must appear exactly once:\n{alpha}"
    );
    assert_eq!(alpha.matches("pub alpha_field").count(), 1);
    assert!(alpha.contains("pub struct Alpha<'a> {"));

    let beta = section(&code, "pub struct Beta");
    assert_eq!(
        beta.matches("pub alpha_field").count(),
        1,
        "cycle-merged field must appear exactly once:\n{beta}"
    );
    assert_eq!(beta.matches("pub beta_field").count(), 1);
}
