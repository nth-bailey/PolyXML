## [0.5.1](https://github.com/nth-bailey/PolyXML/compare/v0.5.0...v0.5.1) (2026-09-12)


### Bug Fixes

* **ci:** fix Homebrew release workflow tag resolution ([9e35fb9](https://github.com/nth-bailey/PolyXML/commit/9e35fb9ed5ea72e613bc6349a30a0fbd6d09388c))
* **ci:** upgrade bump-homebrew-formula-action v3 → v4 ([7f863c4](https://github.com/nth-bailey/PolyXML/commit/7f863c45273fa45e99bd53f34ac5e49a7f4c9d83))

# [0.5.0](https://github.com/nth-bailey/PolyXML/compare/v0.4.0...v0.5.0) (2026-09-12)


### Features

* **binary:** add zero-GIL binary serialization (dumps_binary, loads_binary) with 100% test coverage ([f3c115a](https://github.com/nth-bailey/PolyXML/commit/f3c115aff96a43821562b5b003c981e364df35e8))

# [0.4.0](https://github.com/nth-bailey/PolyXML/compare/v0.3.0...v0.4.0) (2026-09-10)


### Features

* upgrade quick-xml to 0.42, criterion to 0.8, and pyo3 to 0.29 ([440bb14](https://github.com/nth-bailey/PolyXML/commit/440bb1489286a0ddd97f04ca8dce3067bac271d8))

# [0.3.0](https://github.com/nth-bailey/PolyXML/compare/v0.2.1...v0.3.0) (2026-09-10)


### Features

* upgrade quick-xml to 0.37, pyo3 to 0.23, criterion to 0.7 and update dependabot rules ([5e25ff5](https://github.com/nth-bailey/PolyXML/commit/5e25ff522542fb258445a03080b9882f3866bfc5))

## [0.2.1](https://github.com/nth-bailey/PolyXML/compare/v0.2.0...v0.2.1) (2026-09-10)


### Bug Fixes

* **python:** make date/time resolution standalone and ignore breaking cargo major updates in dependabot ([6d04cc8](https://github.com/nth-bailey/PolyXML/commit/6d04cc8d302f2e876152611ffd48014d9ff089d5))

# [0.2.0](https://github.com/nth-bailey/PolyXML/compare/v0.1.0...v0.2.0) (2026-09-10)


### Bug Fixes

* **ci:** make PyPI publish idempotent with skip-existing ([e5c2b32](https://github.com/nth-bailey/PolyXML/commit/e5c2b32459ce54f6f0792d69b85a1be0105f4e1e))


### Features

* **conan:** add Conan 2.0 recipe, ConanCenter package files, and test package ([325cf72](https://github.com/nth-bailey/PolyXML/commit/325cf72b155df356c51800d57139636f8ec4bc2e))
* **packaging:** add AUR PKGBUILD, .SRCINFO, and update packaging recipes ([11a8446](https://github.com/nth-bailey/PolyXML/commit/11a84460595be39f1a23e05fdac17cdf44f97d1a))
* **packaging:** add conda-forge, vcpkg, and Homebrew distribution packages ([c71e3fb](https://github.com/nth-bailey/PolyXML/commit/c71e3fbb1bd9e34ac87e8f2d845e691f1e0b2fe7))
* **perf:** implement streaming iterparse and bridge optimizations ([df5fff9](https://github.com/nth-bailey/PolyXML/commit/df5fff936a6f50eb0c5c90e476585cdf8abab5b8))
* **python:** add rich type support, dataclass kw_only, and stream sources parity with pyxsdata-core ([0700104](https://github.com/nth-bailey/PolyXML/commit/0700104d815c621b0f310f313352a4f37c4b80f3))
