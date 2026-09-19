use polyxml::{PolyXmlError, Result};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct CardDetails<'a> {
    pub number: Cow<'a, str>,
}

impl<'a> CardDetails<'a> {
    pub fn decode_xml(reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut number = None;
        loop {
            match reader.read_event()? {
                Event::Start(e) => {
                    if e.local_name().as_ref() == "number" {
                        let mut text = Cow::Borrowed("");
                        loop {
                            match reader.read_event()? {
                                Event::Text(t) => {
                                    text = match t.into_inner() {
                                        Cow::Borrowed(b) => match quick_xml::escape::unescape(b)? {
                                            Cow::Borrowed(s) => Cow::Borrowed(s),
                                            Cow::Owned(s) => Cow::Owned(s),
                                        },
                                        Cow::Owned(s) => Cow::Owned(
                                            quick_xml::escape::unescape(&s)?.into_owned(),
                                        ),
                                    };
                                }
                                Event::End(end) if end.local_name().as_ref() == "number" => break,
                                _ => {}
                            }
                        }
                        number = Some(text);
                    }
                }
                Event::End(e) if e.local_name().as_ref() == start.local_name().as_ref() => break,
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(Self {
            number: number.unwrap_or(Cow::Borrowed("")),
        })
    }

    pub fn encode_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag_name: Option<&str>,
    ) -> Result<()> {
        let tag = tag_name.unwrap_or("CardDetails");
        writer.write_event(Event::Start(BytesStart::new(tag)))?;
        writer.write_event(Event::Start(BytesStart::new("number")))?;
        writer.write_event(Event::Text(BytesText::new(&self.number)))?;
        writer.write_event(Event::End(BytesEnd::new("number")))?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PaymentChoice<'a> {
    Card(CardDetails<'a>),
    Cash(i32),
}

impl<'a> PaymentChoice<'a> {
    pub fn decode_xml(reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self> {
        match start.local_name().as_ref() {
            "Card" => Ok(PaymentChoice::Card(CardDetails::decode_xml(reader, start)?)),
            "Cash" => {
                let mut text = Cow::Borrowed("");
                loop {
                    match reader.read_event()? {
                        Event::Text(t) => {
                            text = match t.into_inner() {
                                Cow::Borrowed(b) => match quick_xml::escape::unescape(b)? {
                                    Cow::Borrowed(s) => Cow::Borrowed(s),
                                    Cow::Owned(s) => Cow::Owned(s),
                                },
                                Cow::Owned(s) => {
                                    Cow::Owned(quick_xml::escape::unescape(&s)?.into_owned())
                                }
                            };
                        }
                        Event::End(end) if end.local_name().as_ref() == "Cash" => break,
                        _ => {}
                    }
                }
                let val =
                    text.trim()
                        .parse::<i32>()
                        .map_err(|_| PolyXmlError::ScalarParseError {
                            field: "Cash".into(),
                            expected: "i32",
                            value: text.to_string(),
                        })?;
                Ok(PaymentChoice::Cash(val))
            }
            other => Err(PolyXmlError::UnexpectedRootElement {
                expected: "Card or Cash".into(),
                actual: other.into(),
            }),
        }
    }

    pub fn encode_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag_name: Option<&str>,
    ) -> Result<()> {
        match self {
            PaymentChoice::Card(ref c) => c.encode_xml(writer, tag_name.or(Some("Card"))),
            PaymentChoice::Cash(ref val) => {
                let tag = tag_name.unwrap_or("Cash");
                writer.write_event(Event::Start(BytesStart::new(tag)))?;
                writer.write_event(Event::Text(BytesText::new(&val.to_string())))?;
                writer.write_event(Event::End(BytesEnd::new(tag)))?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Order<'a> {
    pub id: i32,
    pub payment: PaymentChoice<'a>,
}

impl<'a> Order<'a> {
    pub fn from_xml(xml: &'a str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event()? {
                Event::Start(e) => return Self::decode_xml(&mut reader, &e),
                Event::Eof => break,
                _ => {}
            }
        }
        Err(PolyXmlError::SchemaError("Unexpected EOF".into()))
    }

    pub fn decode_xml(reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut id = None;
        let mut payment = None;

        for attr in start.attributes() {
            let attr = attr?;
            if attr.key.local_name().as_ref() == "id" {
                let s = attr.value.as_ref();
                id = Some(
                    s.parse::<i32>()
                        .map_err(|_| PolyXmlError::ScalarParseError {
                            field: "id".into(),
                            expected: "i32",
                            value: s.into(),
                        })?,
                );
            }
        }

        loop {
            match reader.read_event()? {
                Event::Start(e) => match e.local_name().as_ref() {
                    "Card" | "Cash" => {
                        let p = PaymentChoice::decode_xml(reader, &e)?;
                        payment = Some(p);
                    }
                    _ => {}
                },
                Event::End(e) if e.local_name().as_ref() == start.local_name().as_ref() => break,
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self {
            id: id.ok_or_else(|| {
                PolyXmlError::SchemaError("Missing required attribute 'id'".into())
            })?,
            payment: payment.ok_or_else(|| {
                PolyXmlError::SchemaError("Missing required choice element 'payment'".into())
            })?,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut writer = Writer::new(std::io::Cursor::new(&mut buf));
        self.encode_xml(&mut writer, None)?;
        Ok(buf)
    }

    pub fn encode_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag_name: Option<&str>,
    ) -> Result<()> {
        let tag = tag_name.unwrap_or("Order");
        let mut start = BytesStart::new(tag);
        let id_str = self.id.to_string();
        start.push_attribute(("id", id_str.as_str()));
        writer.write_event(Event::Start(start))?;
        self.payment.encode_xml(writer, None)?;
        writer.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }
}

#[test]
fn test_order_choice_roundtrip() {
    let xml = "<Order id=\"99\"><Card><number>1234-5678</number></Card></Order>";
    let order = Order::from_xml(xml).unwrap();
    assert_eq!(order.id, 99);
    match order.payment {
        PaymentChoice::Card(ref c) => {
            assert_eq!(c.number, "1234-5678");
            assert!(matches!(c.number, Cow::Borrowed(_)));
        }
        _ => panic!("Expected Card variant"),
    }

    let serialized = String::from_utf8(order.to_xml().unwrap()).unwrap();
    assert_eq!(serialized, xml);

    // Also test Cash variant
    let xml_cash = "<Order id=\"100\"><Cash>50</Cash></Order>";
    let order_cash = Order::from_xml(xml_cash).unwrap();
    assert_eq!(order_cash.id, 100);
    match order_cash.payment {
        PaymentChoice::Cash(v) => assert_eq!(v, 50),
        _ => panic!("Expected Cash variant"),
    }
    let serialized_cash = String::from_utf8(order_cash.to_xml().unwrap()).unwrap();
    assert_eq!(serialized_cash, xml_cash);
}

// -----------------------------------------------------------------------------
// Test 2: Facet validation (minLength / maxLength)
// -----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct Username<'a> {
    pub value: Cow<'a, str>,
}

impl<'a> Username<'a> {
    pub fn from_xml(xml: &'a str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event()? {
                Event::Start(e) => return Self::decode_xml(&mut reader, &e),
                Event::Eof => break,
                _ => {}
            }
        }
        Err(PolyXmlError::SchemaError("Unexpected EOF".into()))
    }

    pub fn decode_xml(reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut text = Cow::Borrowed("");
        loop {
            match reader.read_event()? {
                Event::Text(t) => {
                    text = match t.into_inner() {
                        Cow::Borrowed(b) => match quick_xml::escape::unescape(b)? {
                            Cow::Borrowed(s) => Cow::Borrowed(s),
                            Cow::Owned(s) => Cow::Owned(s),
                        },
                        Cow::Owned(s) => Cow::Owned(quick_xml::escape::unescape(&s)?.into_owned()),
                    };
                }
                Event::End(e) if e.local_name().as_ref() == start.local_name().as_ref() => break,
                Event::Eof => break,
                _ => {}
            }
        }

        // Facet: min_length: 3, max_length: 10
        if text.chars().count() < 3 {
            return Err(PolyXmlError::FacetViolation {
                field: "Username".into(),
                expected: "minLength >= 3".into(),
                actual: format!("length {}", text.chars().count()),
            });
        }
        if text.chars().count() > 10 {
            return Err(PolyXmlError::FacetViolation {
                field: "Username".into(),
                expected: "maxLength <= 10".into(),
                actual: format!("length {}", text.chars().count()),
            });
        }

        Ok(Self { value: text })
    }
}

#[test]
fn test_facet_validation_min_max_length() {
    let valid = "<Username>alice</Username>";
    let u = Username::from_xml(valid).unwrap();
    assert_eq!(u.value, "alice");
    assert!(matches!(u.value, Cow::Borrowed(_)));

    let too_short = "<Username>al</Username>";
    match Username::from_xml(too_short) {
        Err(PolyXmlError::FacetViolation {
            field, expected, ..
        }) => {
            assert_eq!(field, "Username");
            assert_eq!(expected, "minLength >= 3");
        }
        other => panic!("Expected FacetViolation, got {:?}", other),
    }

    let too_long = "<Username>supercalifragilistic</Username>";
    match Username::from_xml(too_long) {
        Err(PolyXmlError::FacetViolation {
            field, expected, ..
        }) => {
            assert_eq!(field, "Username");
            assert_eq!(expected, "maxLength <= 10");
        }
        other => panic!("Expected FacetViolation, got {:?}", other),
    }
}

// -----------------------------------------------------------------------------
// Test 3: Empty / Self-Closing elements
// -----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct TagItem<'a> {
    pub id: i32,
    pub label: Cow<'a, str>,
}

impl<'a> TagItem<'a> {
    pub fn from_xml(xml: &'a str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event()? {
                Event::Start(e) => return Self::decode_xml(&mut reader, &e),
                Event::Empty(e) => return Self::decode_xml_empty(&e),
                Event::Eof => break,
                _ => {}
            }
        }
        Err(PolyXmlError::SchemaError("Unexpected EOF".into()))
    }

    pub fn decode_xml(_reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self> {
        Self::decode_xml_empty(start)
    }

    pub fn decode_xml_empty(start: &BytesStart<'_>) -> Result<Self> {
        let mut id = None;
        let mut label = None;

        for attr in start.attributes() {
            let attr = attr?;
            match attr.key.local_name().as_ref() {
                "id" => {
                    let s = attr.value.as_ref();
                    id = Some(
                        s.parse::<i32>()
                            .map_err(|_| PolyXmlError::ScalarParseError {
                                field: "id".into(),
                                expected: "i32",
                                value: s.into(),
                            })?,
                    );
                }
                "label" => {
                    let text = match quick_xml::escape::unescape(attr.value.as_ref())? {
                        Cow::Borrowed(s) => Cow::Owned(s.to_string()),
                        Cow::Owned(s) => Cow::Owned(s),
                    };
                    label = Some(text);
                }
                _ => {}
            }
        }

        Ok(Self {
            id: id.ok_or_else(|| {
                PolyXmlError::SchemaError("Missing required attribute 'id'".into())
            })?,
            label: label.unwrap_or(Cow::Borrowed("")),
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut writer = Writer::new(std::io::Cursor::new(&mut buf));
        self.encode_xml(&mut writer, None)?;
        Ok(buf)
    }

    pub fn encode_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag_name: Option<&str>,
    ) -> Result<()> {
        let tag = tag_name.unwrap_or("TagItem");
        let mut start = BytesStart::new(tag);
        let id_str = self.id.to_string();
        start.push_attribute(("id", id_str.as_str()));
        start.push_attribute(("label", self.label.as_ref()));
        writer.write_event(Event::Empty(start))?;
        Ok(())
    }
}

#[test]
fn test_empty_element_codec() {
    let xml = "<TagItem id=\"42\" label=\"electronics\"/>";
    let item = TagItem::from_xml(xml).unwrap();
    assert_eq!(item.id, 42);
    assert_eq!(item.label, "electronics");
    assert!(matches!(item.label, Cow::Owned(_)));

    let serialized = String::from_utf8(item.to_xml().unwrap()).unwrap();
    assert_eq!(serialized, xml);
}

// -----------------------------------------------------------------------------
// Test 4: Recursive Boxed type decoding
// -----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct TreeNode<'a> {
    pub value: Cow<'a, str>,
    pub left: Option<Box<TreeNode<'a>>>,
    pub right: Option<Box<TreeNode<'a>>>,
}

impl<'a> TreeNode<'a> {
    pub fn from_xml(xml: &'a str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event()? {
                Event::Start(e) => return Self::decode_xml(&mut reader, &e),
                Event::Eof => break,
                _ => {}
            }
        }
        Err(PolyXmlError::SchemaError("Unexpected EOF".into()))
    }

    pub fn decode_xml(reader: &mut Reader<&'a [u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut value = None;
        let mut left = None;
        let mut right = None;

        loop {
            match reader.read_event()? {
                Event::Start(e) => match e.local_name().as_ref() {
                    "value" => {
                        let mut text = Cow::Borrowed("");
                        loop {
                            match reader.read_event()? {
                                Event::Text(t) => {
                                    text = match t.into_inner() {
                                        Cow::Borrowed(b) => match quick_xml::escape::unescape(b)? {
                                            Cow::Borrowed(s) => Cow::Borrowed(s),
                                            Cow::Owned(s) => Cow::Owned(s),
                                        },
                                        Cow::Owned(s) => Cow::Owned(
                                            quick_xml::escape::unescape(&s)?.into_owned(),
                                        ),
                                    };
                                }
                                Event::End(end) if end.local_name().as_ref() == "value" => break,
                                _ => {}
                            }
                        }
                        value = Some(text);
                    }
                    "left" => loop {
                        match reader.read_event()? {
                            Event::Start(child) if child.local_name().as_ref() == "TreeNode" => {
                                left = Some(Box::new(TreeNode::decode_xml(reader, &child)?));
                            }
                            Event::End(end) if end.local_name().as_ref() == "left" => break,
                            _ => {}
                        }
                    },
                    "right" => loop {
                        match reader.read_event()? {
                            Event::Start(child) if child.local_name().as_ref() == "TreeNode" => {
                                right = Some(Box::new(TreeNode::decode_xml(reader, &child)?));
                            }
                            Event::End(end) if end.local_name().as_ref() == "right" => break,
                            _ => {}
                        }
                    },
                    _ => {}
                },
                Event::End(e) if e.local_name().as_ref() == start.local_name().as_ref() => break,
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self {
            value: value.unwrap_or(Cow::Borrowed("")),
            left,
            right,
        })
    }

    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut writer = Writer::new(std::io::Cursor::new(&mut buf));
        self.encode_xml(&mut writer, None)?;
        Ok(buf)
    }

    pub fn encode_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag_name: Option<&str>,
    ) -> Result<()> {
        let tag = tag_name.unwrap_or("TreeNode");
        writer.write_event(Event::Start(BytesStart::new(tag)))?;
        writer.write_event(Event::Start(BytesStart::new("value")))?;
        writer.write_event(Event::Text(BytesText::new(&self.value)))?;
        writer.write_event(Event::End(BytesEnd::new("value")))?;

        if let Some(ref l) = self.left {
            writer.write_event(Event::Start(BytesStart::new("left")))?;
            l.encode_xml(writer, None)?;
            writer.write_event(Event::End(BytesEnd::new("left")))?;
        }
        if let Some(ref r) = self.right {
            writer.write_event(Event::Start(BytesStart::new("right")))?;
            r.encode_xml(writer, None)?;
            writer.write_event(Event::End(BytesEnd::new("right")))?;
        }

        writer.write_event(Event::End(BytesEnd::new(tag)))?;
        Ok(())
    }
}

#[test]
fn test_recursive_boxing_roundtrip() {
    let xml = "<TreeNode><value>root</value><left><TreeNode><value>child</value></TreeNode></left></TreeNode>";
    let tree = TreeNode::from_xml(xml).unwrap();
    assert_eq!(tree.value, "root");
    assert!(tree.left.is_some());
    assert_eq!(tree.left.as_ref().unwrap().value, "child");
    assert!(tree.right.is_none());

    let serialized = String::from_utf8(tree.to_xml().unwrap()).unwrap();
    assert_eq!(serialized, xml);
}
