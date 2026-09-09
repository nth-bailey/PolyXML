# PolyXML Conan Package & ConanCenter Submission Guide

This directory contains the official Conan 2.0 recipe configuration for **PolyXML**.

---

## 1. Local Testing & Creation

To test or build the PolyXML Conan package locally:

```bash
# 1. Ensure Conan 2.0 is installed
pip install "conan>=2.0"

# 2. Test and package PolyXML from repository root
conan create . --version 0.1.0

# 3. Or test the ConanCenter recipe directly
conan create conan/recipes/polyxml/all --version 0.1.0
```

---

## 2. Consuming PolyXML in C++ via Conan

In your C++ application's `conanfile.txt`:

```ini
[requires]
polyxml/0.1.0

[generators]
CMakeDeps
CMakeToolchain
```

In your `CMakeLists.txt`:

```cmake
find_package(polyxml REQUIRED)
target_link_libraries(my_target PRIVATE polyxml::polyxml)
```

---

## 3. Submitting to ConanCenter (Official Public Registry)

ConanCenter hosts C/C++ packages publicly on [conan.io/center](https://conan.io/center).

### Submission Steps:

1. **Fork** [`conan-io/conan-center-index`](https://github.com/conan-io/conan-center-index) on GitHub.
2. **Clone your fork**:
   ```bash
   git clone https://github.com/<your-username>/conan-center-index.git
   cd conan-center-index
   git checkout -b add-polyxml
   ```
3. **Copy the recipe**:
   Copy the `conan/recipes/polyxml/` folder into `recipes/polyxml/` in the cloned repo:
   ```bash
   cp -r /path/to/PolyXML/conan/recipes/polyxml recipes/
   ```
4. **Test with ConanCenter linter / test**:
   ```bash
   conan create recipes/polyxml/all --version 0.1.0
   ```
5. **Commit and open a Pull Request**:
   ```bash
   git add recipes/polyxml
   git commit -m "feat(recipes): add polyxml 0.1.0"
   git push origin add-polyxml
   ```
   Open a PR against `conan-io/conan-center-index`.
6. **Automated CI**:
   The Conan Center Bot (C3I) will automatically trigger builds across Windows, Linux, and macOS. Once maintainers review and merge, `polyxml/0.1.0` will be live in ConanCenter worldwide!
