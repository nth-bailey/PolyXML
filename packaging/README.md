# PolyXML Distribution & Packaging Center

This directory contains package definitions and recipes for distributing **PolyXML** across enterprise, C/C++, and operating system package managers.

---

## 1. Conda-Forge (`conda-forge`)

Enterprise, Data Engineering, and Scientific Python distribution.

- **Recipe Location**: `packaging/conda-forge/recipe/meta.yaml`
- **Target Repository**: [`conda-forge/staged-recipes`](https://github.com/conda-forge/staged-recipes)

### Submission Steps:
1. Fork `https://github.com/conda-forge/staged-recipes`
2. Create a branch: `git checkout -b add-polyxml`
3. Copy `packaging/conda-forge/recipe/` to `recipes/polyxml/` in `staged-recipes`
4. Commit and open a PR:
   ```bash
   git add recipes/polyxml
   git commit -m "feat(recipes): add polyxml 0.1.0"
   git push origin add-polyxml
   ```
5. Once merged, Conda-Forge bot creates the `conda-forge/polyxml-feedstock` repository and publishes packages for Linux, macOS, and Windows.

---

## 2. Microsoft vcpkg (`vcpkg`)

The premier C/C++ package manager for Windows, MSVC, Visual Studio, and CMake.

- **Port Location**: `packaging/vcpkg/ports/polyxml/`
- **Target Repository**: [`microsoft/vcpkg`](https://github.com/microsoft/vcpkg)

### Submission Steps:
1. Fork `https://github.com/microsoft/vcpkg`
2. Create a branch: `git checkout -b add-polyxml`
3. Copy `packaging/vcpkg/ports/polyxml/` to `ports/polyxml/` in `vcpkg`
4. Commit and open a PR against `microsoft/vcpkg`:
   ```bash
   git add ports/polyxml
   git commit -m "[polyxml] Add port polyxml 0.1.0"
   git push origin add-polyxml
   ```
5. Once merged, developers can install via:
   ```bash
   vcpkg install polyxml
   ```

---

## 3. Homebrew Formula (macOS & Linux)

Package manager for macOS and desktop Linux developers.

- **Formula Location**: `packaging/homebrew/Formula/polyxml.rb`

### Option A: Official Homebrew Tap (Instant)
1. Create a GitHub repository named `homebrew-polyxml` under your account (`nth-bailey/homebrew-polyxml`).
2. Place `packaging/homebrew/Formula/polyxml.rb` into `Formula/polyxml.rb` in that repo.
3. Anyone can immediately install via:
   ```bash
   brew install nth-bailey/polyxml/polyxml
   ```

### Option B: Submit to `Homebrew/homebrew-core`
1. Test locally: `brew audit --strict --online Formula/polyxml.rb`
2. Submit PR to `Homebrew/homebrew-core` via `brew bump-formula-pr` or standard Git PR.
3. Users can then install directly via:
   ```bash
   brew install polyxml
   ```
