---
title: Java 22 Guide (Project Panama FFI)
description: Ultra-fast Java XML data binding using Java 22 Foreign Function & Memory API.
---

# Java Guide (Project Panama)

PolyXML uses Java 22+ **Project Panama (Foreign Function & Memory API)** for zero-JNI, off-heap native bindings.

## Maven Dependency

```xml
<dependency>
    <groupId>io.github.nth-bailey</groupId>
    <artifactId>polyxml</artifactId>
    <version>0.1.0</version>
</dependency>
```

## Example Usage

```java
import io.polyxml.PolyXML;

public class Main {
    public static void main(String[] args) {
        try (var schema = new PolyXML.SchemaBuilder("Server")
                .addField("id", "id", PolyXML.FieldKind.ATTRIBUTE, PolyXML.ScalarType.INT)
                .addField("hostname", "hostname", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.STRING)
                .build()) {

            System.out.println("PolyXML Native Engine Version: " + PolyXML.version());
        }
    }
}
```
