---
title: Java 22 Guide (Project Panama FFI)
description: Ultra-fast Java XML processing using Java 22 Foreign Function & Memory API (Project Panama) with zero-JNI overhead.
---

# Java 22 Guide: Project Panama FFI

PolyXML provides native C/Rust XML data-binding for the modern Java Virtual Machine using **Java 22+ Project Panama (Foreign Function & Memory API - JEP 454)**. It completely eliminates legacy JNI glue code, GC object pinning, and JNI transition overheads by leveraging native off-heap memory and downcall method handles.

---

## 📦 Build Configuration

### Maven (`pom.xml`)

```xml
<dependencies>
    <dependency>
        <groupId>io.github.nth-bailey</groupId>
        <artifactId>polyxml</artifactId>
        <version>0.1.0</version>
    </dependency>
</dependencies>

<build>
    <plugins>
        <plugin>
            <groupId>org.apache.maven.plugins</groupId>
            <artifactId>maven-compiler-plugin</artifactId>
            <version>3.13.0</version>
            <configuration>
                <release>22</release>
                <compilerArgs>
                    <arg>--enable-preview</arg>
                </compilerArgs>
            </configuration>
        </plugin>
    </plugins>
</build>
```

### JVM Runtime Flags

Because Project Panama accesses native off-heap memory, your application requires the `--enable-native-access` JVM flag at runtime:

```bash
java --enable-native-access=ALL-UNNAMED -jar target/my-app.jar
```

---

## 1. Schema Construction & Resource Management

PolyXML schemas are native off-heap objects. `PolyXML.Schema` implements `AutoCloseable`, enabling clean deterministic cleanup with Java's standard `try-with-resources`:

```java
package com.example;

import io.polyxml.PolyXML;

public class Application {
    public static void main(String[] args) {
        System.out.println("PolyXML Native Core Version: " + PolyXML.version());

        // Build an off-heap native schema using try-with-resources
        try (PolyXML.Schema schema = new PolyXML.SchemaBuilder("ServerMetrics")
                .addField("serverId", "id", PolyXML.FieldKind.ATTRIBUTE, PolyXML.ScalarType.INT)
                .addField("hostname", "host", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.STRING)
                .addField("cpuUtilization", "cpu", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.FLOAT)
                .addField("isHealthy", "healthy", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.BOOL)
                .build()) {

            System.out.println("Schema created successfully in off-heap memory.");
        }
    }
}
```

---

## 2. End-to-End Deserialization & Serialization

Deserialize XML strings or byte buffers into native `PolyXML.Value` objects and serialize back to XML:

```java
package com.example;

import io.polyxml.PolyXML;

public class SerializationExample {
    public static void main(String[] args) {
        try (PolyXML.Schema schema = new PolyXML.SchemaBuilder("Sensor")
                .addField("id", "id", PolyXML.FieldKind.ATTRIBUTE, PolyXML.ScalarType.INT)
                .addField("name", "name", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.STRING)
                .addField("reading", "reading", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.FLOAT)
                .addField("calibrated", "calibrated", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.BOOL)
                .build()) {

            String xml = "<Sensor id=\"101\"><name>Barometer</name><reading>1013.25</reading><calibrated>true</calibrated></Sensor>";

            // 1. Deserialize off-heap
            try (PolyXML.Value val = PolyXML.deserialize(xml, schema)) {
                long id = val.getField("id").getInt().orElse(0L);
                String name = val.getField("name").getString().orElse("");
                double reading = val.getField("reading").getFloat().orElse(0.0);
                boolean calibrated = val.getField("calibrated").getBool().orElse(false);

                System.out.printf("Sensor %d [%s]: %.2f (Calibrated: %b)%n", id, name, reading, calibrated);

                // 2. Serialize back to XML string with 2-space indentation
                String outputXml = PolyXML.serializeToString("Sensor", val, schema, 2);
                System.out.println("Output XML:\n" + outputXml);
            }
        }
    }
}
```

---

## 3. XML Namespaces & Prefix Mapping

Declare model and field namespaces, and serialize with custom prefix mappings:

```java
package com.example;

import io.polyxml.PolyXML;
import java.util.Map;

public class NamespaceExample {
    public static void main(String[] args) {
        try (PolyXML.Schema schema = new PolyXML.SchemaBuilder("Order")
                .setNamespace("https://example.com/orders")
                .addField("id", "id", PolyXML.FieldKind.ATTRIBUTE, PolyXML.ScalarType.INT)
                .addField("item", "item", PolyXML.FieldKind.ELEMENT, PolyXML.ScalarType.STRING, "https://example.com/items")
                .build()) {

            String xml = "<ns0:Order xmlns:ns0=\"https://example.com/orders\" xmlns:ns1=\"https://example.com/items\" id=\"888\"><ns1:item>JavaGadget</ns1:item></ns0:Order>";

            try (PolyXML.Value val = PolyXML.deserialize(xml, schema)) {
                long id = val.getField("id").getInt().orElse(0L);
                String item = val.getField("item").getString().orElse("");
                System.out.printf("Order #%d: %s%n", id, item);

                // Serialize with custom prefix mapping
                Map<String, String> nsMap = Map.of(
                    "ord", "https://example.com/orders",
                    "itm", "https://example.com/items"
                );

                byte[] bytes = PolyXML.serializeWithOptions("Order", val, schema, 2, true, nsMap);
                System.out.println(new String(bytes, java.nio.charset.StandardCharsets.UTF_8));
            }
        }
    }
}
```

---

## 4. Off-Heap Confined Arenas & Zero-GC Pressure

When passing large XML strings or byte streams from Java into PolyXML, Java 22's `Arena.ofConfined()` allocates off-heap memory segments that bypass the JVM garbage collector entirely:

```java
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.nio.charset.StandardCharsets;

public class OffHeapBufferExample {
    public static void allocateAndPassXml(String xmlData) {
        // Arena confines allocation to current thread and frees immediately upon exit
        try (Arena arena = Arena.ofConfined()) {
            byte[] xmlBytes = xmlData.getBytes(StandardCharsets.UTF_8);
            MemorySegment nativeBuffer = arena.allocate(xmlBytes.length);
            nativeBuffer.copyFrom(MemorySegment.ofArray(xmlBytes));

            System.out.printf("Allocated %d bytes off-heap with zero GC pressure%n", nativeBuffer.byteSize());
            // Pass nativeBuffer.address() to PolyXML native routines...
        } // Instant off-heap deallocation occurs here
    }
}
```

---

## 5. Enterprise Architecture: ISO 20022 Batch Processing

In high-throughput enterprise architectures (e.g. processing millions of ISO 20022 XML financial payment messages or HL7 clinical records):

1. **Singleton Native Schemas**: Store `PolyXML.Schema` instances in `static final` fields or Spring Singleton beans so they are created once at application startup.
2. **Eliminate Garbage Collection Pauses**: By streaming raw socket or file bytes into off-heap `MemorySegment` buffers, you prevent millions of short-lived XML DOM strings from exhausting the JVM Young Generation heap.
3. **Thread Safety**: PolyXML's native schema handles are immutable and read-only after construction, making them safe to share concurrently across all JVM virtual threads (Project Loom).

