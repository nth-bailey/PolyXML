---
title: Multi-Language Quickstart
description: Get started with PolyXML in Rust, Python, C++, Go, Java, or TypeScript in under 5 minutes.
---

# Multi-Language Quickstart

Choose your preferred language to see how PolyXML deserializes XML payloads into strongly-typed models:

=== "Python"

    ### Installation
    ```bash
    pip install polyxml
    ```

    ### Example
    ```python
    from dataclasses import dataclass, field
    import polyxml

    @dataclass
    class Sensor:
        id: int = field(metadata={"type": "Attribute"})
        name: str = field(metadata={"type": "Element"})
        reading: float = field(metadata={"type": "Element"})
        calibrated: bool = field(metadata={"type": "Element"})

    xml = b"""
    <Sensor id="101">
        <name>Barometric Altimeter</name>
        <reading>1013.25</reading>
        <calibrated>true</calibrated>
    </Sensor>
    """

    # 1. Deserialize XML directly into dataclass
    sensor = polyxml.deserialize(xml, Sensor)
    print(f"Sensor: {sensor.name}, Reading: {sensor.reading}")

    # 2. Serialize model back to formatted XML
    xml_output = polyxml.serialize(sensor, indent=2)
    print(xml_output.decode("utf-8"))

    # 3. Stream multi-gigabyte XML files in O(1) memory
    # for s in polyxml.iterparse(open("huge.xml", "rb").read(), Sensor, tag="Sensor"):
    #     print(s.name)

    # 4. Zero-GIL binary serialization for key-value DBs (MDBX, Redis) & IPC
    binary_blob = polyxml.dumps_binary(sensor)
    restored = polyxml.loads_binary(binary_blob, Sensor)
    assert restored.name == sensor.name
    ```

=== "Rust"

    ### Cargo Dependency
    ```toml
    [dependencies]
    polyxml = "0.1"
    ```

    ### Example
    ```rust
    use std::sync::Arc;
    use polyxml::schema::{ModelSchema, FieldSchema, FieldKind, ScalarType, ValueType};
    use polyxml::{deserialize, serialize};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let schema = ModelSchema::builder("Sensor")
            .field(FieldSchema::new("id", b"id", FieldKind::Attribute, ValueType::Scalar(ScalarType::Int)))
            .field(FieldSchema::new("name", b"name", FieldKind::Element, ValueType::Scalar(ScalarType::String)))
            .field(FieldSchema::new("reading", b"reading", FieldKind::Element, ValueType::Scalar(ScalarType::Float)))
            .field(FieldSchema::new("calibrated", b"calibrated", FieldKind::Element, ValueType::Scalar(ScalarType::Bool)))
            .build();

        let xml = br#"<Sensor id="101"><name>Barometric Altimeter</name><reading>1013.25</reading><calibrated>true</calibrated></Sensor>"#;

        // 1. Deserialize
        let val = deserialize(xml, Arc::clone(&schema))?;
        println!("Name: {}", val.get("name").unwrap().as_str().unwrap());

        // 2. Serialize
        let output = serialize("Sensor", &val, &schema, Some(2))?;
        println!("{}", std::str::from_utf8(&output)?);

        Ok(())
    }
    ```

=== "Modern C++20"

    ### Include Header
    ```cpp
    #include "polyxml.hpp"
    #include <iostream>

    int main() {
        auto schema = polyxml::SchemaBuilder("Sensor")
            .add_attribute("id", "id", POLYXML_SCALAR_INT)
            .add_element("name", "name", POLYXML_SCALAR_STRING)
            .add_element("reading", "reading", POLYXML_SCALAR_FLOAT)
            .add_element("calibrated", "calibrated", POLYXML_SCALAR_BOOL)
            .build();

        std::string xml = R"(<Sensor id="101"><name>Gyro</name><reading>99.5</reading><calibrated>true</calibrated></Sensor>)";

        // 1. Deserialize
        auto val = polyxml::deserialize(xml, schema);
        std::cout << "Name: " << val.get("name")->as_string().value() << "\n";

        // 2. Serialize
        std::string output = polyxml::serialize("Sensor", val, schema, 2);
        std::cout << output << "\n";

        return 0;
    }
    ```

=== "Go"

    ### Go Import
    ```go
    import (
        "fmt"
        "github.com/nth-bailey/PolyXML/bindings/go"
    )

    func main() {
        builder, _ := polyxml.NewSchemaBuilder("Sensor")
        builder.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
        builder.AddField("name", "name", polyxml.FieldElement, polyxml.ScalarString)
        builder.AddField("reading", "reading", polyxml.FieldElement, polyxml.ScalarFloat)
        schema, _ := builder.Build()

        xml := []byte(`<Sensor id="101"><name>Altimeter</name><reading>1013.25</reading></Sensor>`)

        // 1. Deserialize
        val, _ := polyxml.Deserialize(xml, schema)
        name, _ := val.GetField("name").GetString()
        fmt.Println("Sensor Name:", name)

        // 2. Serialize
        output, _ := polyxml.Serialize("Sensor", val, schema, 2)
        fmt.Println(string(output))
    }
    ```

=== "TypeScript & Node.js"

    ### NPM Install
    ```bash
    npm install polyxml
    ```

    ### Example
    ```typescript
    import { deserialize, serialize, ModelSchema } from 'polyxml';

    const schema: ModelSchema = {
      name: 'Sensor',
      fields: [
        { name: 'id', xmlName: 'id', kind: 'attribute', scalarType: 'int' },
        { name: 'name', xmlName: 'name', kind: 'element', scalarType: 'string' },
        { name: 'reading', xmlName: 'reading', kind: 'element', scalarType: 'float' },
        { name: 'calibrated', xmlName: 'calibrated', kind: 'element', scalarType: 'bool' },
      ],
    };

    const xml = '<Sensor id="101"><name>Barometric</name><reading>1013.25</reading><calibrated>true</calibrated></Sensor>';

    // 1. Deserialize
    const sensor = deserialize(xml, schema);
    console.log(`Sensor: ${sensor.name}, Reading: ${sensor.reading}`);

    // 2. Serialize
    const output = serialize('Sensor', sensor, schema, 2);
    console.log(new TextDecoder().decode(output));
    ```

=== "Java (Panama FFI)"

    ### Maven Dependency
    ```xml
    <dependency>
        <groupId>io.github.nth-bailey</groupId>
        <artifactId>polyxml</artifactId>
        <version>0.1.0</version>
    </dependency>
    ```

    ### Example
    ```java
    import io.polyxml.PolyXML;

    public class App {
        public static void main(String[] args) {
            try (var schema = new PolyXML.SchemaBuilder("Sensor")
                    .addField("id", "id", PolyXML.FieldKind.ATTRIBUTE, PolyXML.ScalarType.INT)
                    .addField("name", "name", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.STRING)
                    .build()) {

                System.out.println("PolyXML Native Version: " + PolyXML.version());
            }
        }
    }
    ```
