<p align="center">
  <a href="https://github.com/polyxml/PolyXML">
    <img src="https://raw.githubusercontent.com/polyxml/PolyXML/main/docs/assets/brand/logo_polyxml_banner.png" alt="PolyXML" width="800">
  </a>
</p>

# PolyXML C++ Bindings

<p align="center">
  <a href="https://en.cppreference.com/w/cpp/20"><img src="https://img.shields.io/badge/C%2B%2B-20%20%7C%2023-00599C.svg?logo=c%2B%2B" alt="C++: 20"></a>
  <a href="https://cmake.org"><img src="https://img.shields.io/badge/CMake-3.20%2B-064F8C.svg?logo=cmake&logoColor=white" alt="CMake: 3.20+"></a>
  <a href="https://polyxml.github.io/PolyXML/languages/cpp/"><img src="https://img.shields.io/badge/docs-zensical-blue.svg" alt="Documentation"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

Modern C++20 header-only and C-ABI runtime bindings for PolyXML.

Provides zero-copy, Xerces-free XML serialization and deserialization using modern C++ idioms: `std::string_view`, `std::optional`, `std::variant`, and C++20 Concepts.

---

## Integration

### CMake (`FetchContent`)

```cmake
include(FetchContent)
FetchContent_Declare(
    polyxml
    GIT_REPOSITORY https://github.com/polyxml/PolyXML.git
    GIT_TAG        main
    SOURCE_SUBDIR  bindings/cpp
)
FetchContent_MakeAvailable(polyxml)

target_link_libraries(my_app PRIVATE polyxml_cpp)
```

---

## Features

- **🚀 Xerces-Free**: Completely eliminates Apache Xerces-C++ and GPL licensing restrictions.
- **✨ Modern C++20**: Pure value types, concepts, and zero raw pointers.
- **⚡ Native Rust Speed**: Calls directly into the Rust streaming engine with minimal memory overhead.

---

## License

MIT
