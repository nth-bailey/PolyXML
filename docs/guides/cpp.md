---
title: Modern C++20 Guide
description: Header-only, zero-overhead C++20 bindings for PolyXML.
---

# Modern C++20 Guide

PolyXML provides a header-only, modern C++20 library in `bindings/cpp/include/polyxml.hpp`.

## CMake Integration

```cmake
add_subdirectory(bindings/cpp)
target_link_libraries(my_app PRIVATE polyxml_cpp)
```

## Example Usage

```cpp
#include "polyxml.hpp"
#include <iostream>

int main() {
    auto schema = polyxml::SchemaBuilder("Mission")
        .add_attribute("id", "id", POLYXML_SCALAR_INT)
        .add_element("objective", "objective", POLYXML_SCALAR_STRING)
        .add_element("active", "active", POLYXML_SCALAR_BOOL)
        .build();

    std::string xml = R"(<Mission id="99"><objective>Recon</objective><active>true</active></Mission>)";

    auto mission = polyxml::deserialize(xml, schema);
    std::cout << "Objective: " << mission.get("objective")->as_string().value() << "\n";

    std::string output = polyxml::serialize("Mission", mission, schema, 2);
    std::cout << output << "\n";
    return 0;
}
```
