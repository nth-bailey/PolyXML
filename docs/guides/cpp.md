---
title: Modern C++20 Guide
description: Header-only, zero-overhead C++20 bindings for PolyXML with RAII, move semantics, and zero memory leaks.
---

# Modern C++20 Guide

PolyXML delivers high-throughput native XML processing for modern C++20 applications. The C++ bindings are header-only (`bindings/cpp/include/polyxml.hpp`), wrapping the pure Rust engine (`polyxml-c`) with idiomatic C++20 types, `std::optional`, `std::string_view`, RAII memory management, and move semantics.

---

## 📦 Build System Integration

### Option A: CMake `FetchContent`

```cmake
include(FetchContent)
FetchContent_Declare(
    polyxml
    GIT_REPOSITORY https://github.com/nth-bailey/PolyXML.git
    GIT_TAG        main
)
FetchContent_MakeAvailable(polyxml)

target_link_libraries(my_app PRIVATE polyxml_cpp)
```

### Option B: CMake `add_subdirectory`

```cmake
add_subdirectory(path/to/PolyXML/bindings/cpp)
target_link_libraries(my_app PRIVATE polyxml_cpp)
```

### Option C: Conan & vcpkg

- **Conan**: Add `polyxml/0.1.0` to your `conanfile.txt`.
- **vcpkg**: Run `vcpkg install polyxml`.

---

## 1. Schema Construction & Basic Deserialization

Define an XML schema using the fluent `SchemaBuilder` API and parse XML into a type-safe `polyxml::Value`:

```cpp
#include "polyxml.hpp"
#include <iostream>

int main() {
    // 1. Build the schema
    auto schema = polyxml::SchemaBuilder("Telemetry")
        .add_attribute("device_id", "id", POLYXML_SCALAR_INT)
        .add_element("altitude", "altitude", POLYXML_SCALAR_FLOAT)
        .add_element("armed", "armed", POLYXML_SCALAR_BOOL)
        .add_element("status", "status", POLYXML_SCALAR_STRING)
        .build();

    std::string_view xml = R"(
        <Telemetry id="1001">
            <altitude>24500.5</altitude>
            <armed>true</armed>
            <status>STABLE</status>
        </Telemetry>
    )";

    // 2. Deserialize XML string into polyxml::Value
    polyxml::Value val = polyxml::deserialize(xml, schema);

    // 3. Extract values using type-safe std::optional accessors
    int64_t id = val.get("device_id")->as_int().value_or(0);
    double alt = val.get("altitude")->as_float().value_or(0.0);
    bool armed = val.get("armed")->as_bool().value_or(false);
    std::string_view status = val.get("status")->as_string().value_or("UNKNOWN");

    std::cout << "Device " << id << " (" << status << "): " << alt << " ft, Armed: " << std::boolalpha << armed << "\n";

    return 0;
}
```

---

## 2. Formatting & Serialization

Convert native values back into formatted XML with customizable indentation:

```cpp
#include "polyxml.hpp"
#include <iostream>

int main() {
    auto schema = polyxml::SchemaBuilder("SystemConfig")
        .add_attribute("env", "env", POLYXML_SCALAR_STRING)
        .add_element("max_threads", "max_threads", POLYXML_SCALAR_INT)
        .add_element("debug", "debug", POLYXML_SCALAR_BOOL)
        .build();

    std::string xml = R"(<SystemConfig env="production"><max_threads>32</max_threads><debug>false</debug></SystemConfig>)";
    auto val = polyxml::deserialize(xml, schema);

    // Serialize with 2-space indentation
    std::string pretty_xml = polyxml::serialize("SystemConfig", val, schema, 2);
    std::cout << "Formatted XML:\n" << pretty_xml << "\n";

    // Compact serialization (indent = 0)
    std::string compact_xml = polyxml::serialize("SystemConfig", val, schema, 0);
    std::cout << "Compact:\n" << compact_xml << "\n";

    return 0;
}
```

---

## 3. RAII, Move Semantics & Exception Safety

`polyxml::Value` and `polyxml::Schema` are strict RAII objects:
- Destruction automatically releases underlying Rust memory (`polyxml_value_free`, `polyxml_schema_free`).
- Copy construction is explicitly disabled to prevent accidental double-free bugs.
- Move construction (`std::move`) transfers ownership with zero overhead.
- Parsing or serialization errors throw `polyxml::Exception`.

```cpp
#include "polyxml.hpp"
#include <iostream>
#include <vector>

void process_batch(const std::vector<std::string>& payloads) {
    auto schema = polyxml::SchemaBuilder("Sensor")
        .add_attribute("id", "id", POLYXML_SCALAR_INT)
        .add_element("val", "val", POLYXML_SCALAR_FLOAT)
        .build();

    for (const auto& payload : payloads) {
        try {
            // Deserializes and manages lifetime via RAII
            polyxml::Value record = polyxml::deserialize(payload, schema);
            
            // Move ownership into another container or function
            polyxml::Value moved_record = std::move(record);
            
            std::cout << "Processed sensor: " << moved_record.get("id")->as_int().value_or(-1) << "\n";
        } catch (const polyxml::Exception& ex) {
            std::cerr << "Malformed XML payload: " << ex.what() << "\n";
        }
    }
}
```

---

## 4. Struct Mapping Pattern

For production codebases, wrap `polyxml::Value` extraction into a typed C++ struct:

```cpp
#include "polyxml.hpp"
#include <iostream>
#include <string>

struct DroneTelemetry {
    int64_t drone_id;
    double battery_pct;
    double velocity_mps;
    std::string flight_mode;

    static DroneTelemetry from_xml(std::string_view xml, const polyxml::Schema& schema) {
        auto val = polyxml::deserialize(xml, schema);
        return DroneTelemetry{
            .drone_id = val.get("id")->as_int().value_or(0),
            .battery_pct = val.get("battery")->as_float().value_or(0.0),
            .velocity_mps = val.get("velocity")->as_float().value_or(0.0),
            .flight_mode = std::string(val.get("mode")->as_string().value_or("MANUAL")),
        };
    }
};

int main() {
    auto schema = polyxml::SchemaBuilder("Drone")
        .add_attribute("id", "id", POLYXML_SCALAR_INT)
        .add_element("battery", "battery", POLYXML_SCALAR_FLOAT)
        .add_element("velocity", "velocity", POLYXML_SCALAR_FLOAT)
        .add_element("mode", "mode", POLYXML_SCALAR_STRING)
        .build();

    std::string xml = R"(<Drone id="707"><battery>94.5</battery><velocity>18.2</velocity><mode>AUTONOMOUS</mode></Drone>)";
    
    DroneTelemetry telem = DroneTelemetry::from_xml(xml, schema);
    std::cout << "Drone " << telem.drone_id << " in " << telem.flight_mode 
              << " mode at " << telem.velocity_mps << " m/s (" << telem.battery_pct << "% battery)\n";

    return 0;
}
```

---

## 5. Performance Guidelines for C++20

1. **Keep `polyxml::Schema` Instances Long-Lived**: Creating a schema involves heap allocation and string parsing. Create schemas once (e.g. as `static const` or class members) and reuse them across all requests.
2. **Use `std::string_view`**: The `polyxml::deserialize` function accepts `std::string_view`, allowing zero-copy parsing from network buffers, memory-mapped files (`mmap`), or string literals.
3. **Avoid Copying String Values**: `val.get("field")->as_string()` returns `std::optional<std::string_view>`, pointing directly into the parsed token memory without heap string allocations.
