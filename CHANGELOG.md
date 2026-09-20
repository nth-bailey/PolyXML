# [0.15.0](https://github.com/nth-bailey/PolyXML/compare/v0.14.3...v0.15.0) (2026-09-20)


### Features

* **codegen/java:** add Jackson backend support for Spring Boot / enterprise serialization ([78fe976](https://github.com/nth-bailey/PolyXML/commit/78fe9767aef864e39d34b7003aef0be8dcc97fae)), closes [#41](https://github.com/nth-bailey/PolyXML/issues/41)

## [0.14.3](https://github.com/nth-bailey/PolyXML/compare/v0.14.2...v0.14.3) (2026-09-20)


### Bug Fixes

* **rust-codegen:** close attribute loop delimiter and match utf-8 attribute keys ([d0e9a87](https://github.com/nth-bailey/PolyXML/commit/d0e9a87c464f992c31a7a2da42456594d773ddbe))
* **rust-codegen:** use attr.key.local_name().as_ref() directly for str matching ([3409c3f](https://github.com/nth-bailey/PolyXML/commit/3409c3f681ae2ff20c3e3e23c7897da1bbea6d24))

## [0.14.2](https://github.com/nth-bailey/PolyXML/compare/v0.14.1...v0.14.2) (2026-09-20)


### Bug Fixes

* **codegen:** topologically sort typescript schemas by field dependencies and fix multiline doc comments ([902a31e](https://github.com/nth-bailey/PolyXML/commit/902a31e98c7de92d95da83bc94a59bbd3914cafc))
* **csharp:** add new keyword to Validate method on derived records to prevent CS0108 ([8533375](https://github.com/nth-bailey/PolyXML/commit/85333754f1efd617102c5f6052e22d6ab6cac1a4))
* **rust:** correct quick-xml 0.42 string matching, FromStr result, and enum default derive ([ccbb0fa](https://github.com/nth-bailey/PolyXML/commit/ccbb0fac9f1e4fe20f3c17fbdea2ba55f7c95251))

## [0.14.1](https://github.com/nth-bailey/PolyXML/compare/v0.14.0...v0.14.1) (2026-09-20)


### Bug Fixes

* **ci:** eliminate concurrent dotnet first-time initialization race in tests ([621668e](https://github.com/nth-bailey/PolyXML/commit/621668e54e94ba3558c542959af6f2efbc299872))

# [0.14.0](https://github.com/nth-bailey/PolyXML/compare/v0.13.0...v0.14.0) (2026-09-20)


### Features

* **transcoder:** dual-format XML ↔ JSON engine, annotations, codecs, and transcoder ([#39](https://github.com/nth-bailey/PolyXML/issues/39)) ([97f9fd0](https://github.com/nth-bailey/PolyXML/commit/97f9fd01fc6684ea889cb4f4d9cdf72d3a658422)), closes [#35](https://github.com/nth-bailey/PolyXML/issues/35) [#35](https://github.com/nth-bailey/PolyXML/issues/35) [#35](https://github.com/nth-bailey/PolyXML/issues/35) [#36](https://github.com/nth-bailey/PolyXML/issues/36) [#35](https://github.com/nth-bailey/PolyXML/issues/35) [#36](https://github.com/nth-bailey/PolyXML/issues/36) [#37](https://github.com/nth-bailey/PolyXML/issues/37) [#37](https://github.com/nth-bailey/PolyXML/issues/37) [#37](https://github.com/nth-bailey/PolyXML/issues/37) [#34](https://github.com/nth-bailey/PolyXML/issues/34) [#35](https://github.com/nth-bailey/PolyXML/issues/35) [#36](https://github.com/nth-bailey/PolyXML/issues/36) [#37](https://github.com/nth-bailey/PolyXML/issues/37)

# [0.13.0](https://github.com/nth-bailey/PolyXML/compare/v0.12.0...v0.13.0) (2026-09-20)


### Features

* **json:** add native JSON serialization and deserialization across PolyXML ([#38](https://github.com/nth-bailey/PolyXML/issues/38)) ([86a855c](https://github.com/nth-bailey/PolyXML/commit/86a855c59f47c0fc61268c3c9e52147463d406fa))

# [0.12.0](https://github.com/nth-bailey/PolyXML/compare/v0.11.1...v0.12.0) (2026-09-20)


### Features

* Next-Gen Polyglot XML Schema Compiler & Toolchain (Epic [#22](https://github.com/nth-bailey/PolyXML/issues/22)) ([#33](https://github.com/nth-bailey/PolyXML/issues/33)) ([ddfc204](https://github.com/nth-bailey/PolyXML/commit/ddfc204ea0d4feb197ed9e380cd193f5f6206455)), closes [#23](https://github.com/nth-bailey/PolyXML/issues/23) [#31](https://github.com/nth-bailey/PolyXML/issues/31) [#24](https://github.com/nth-bailey/PolyXML/issues/24) [#25](https://github.com/nth-bailey/PolyXML/issues/25) [#32](https://github.com/nth-bailey/PolyXML/issues/32) [#28](https://github.com/nth-bailey/PolyXML/issues/28) [#27](https://github.com/nth-bailey/PolyXML/issues/27) [#26](https://github.com/nth-bailey/PolyXML/issues/26) [#29](https://github.com/nth-bailey/PolyXML/issues/29) [#30](https://github.com/nth-bailey/PolyXML/issues/30)

## [0.11.1](https://github.com/nth-bailey/PolyXML/compare/v0.11.0...v0.11.1) (2026-09-17)


### Bug Fixes

* **core:** track unknown element subtree depth and strip duplicate prefixes in qualified serialization ([ee36649](https://github.com/nth-bailey/PolyXML/commit/ee366494739b14f00ad13a43f1d3a04864b3db6e))

# [0.11.0](https://github.com/nth-bailey/PolyXML/compare/v0.10.0...v0.11.0) (2026-09-17)


### Features

* multi-language feature parity for W3C XML namespaces across C, C++, Go, Node, and Java ([8965862](https://github.com/nth-bailey/PolyXML/commit/896586258ca535c36c1750d1ddbd8fa755369d8e))

# [0.10.0](https://github.com/nth-bailey/PolyXML/compare/v0.9.0...v0.10.0) (2026-09-17)


### Features

* **core,python:** implement toggleable W3C XML namespace support and prefix mapping ([fba695e](https://github.com/nth-bailey/PolyXML/commit/fba695e0a3697d6978c4b52738158ebff453407e))

# [0.9.0](https://github.com/nth-bailey/PolyXML/compare/v0.8.1...v0.9.0) (2026-09-13)


### Features

* add context7 configuration file ([5e01fd6](https://github.com/nth-bailey/PolyXML/commit/5e01fd689c7bad393251e6d523586a767782519e))

## [0.8.1](https://github.com/nth-bailey/PolyXML/compare/v0.8.0...v0.8.1) (2026-09-13)


### Bug Fixes

* **ci:** deduplicate release dispatch and handle concurrent maven central deployments ([7f92648](https://github.com/nth-bailey/PolyXML/commit/7f9264899e2800aa5f5b85904b892fa6c9192b8b))

# [0.8.0](https://github.com/nth-bailey/PolyXML/compare/v0.7.0...v0.8.0) (2026-09-13)


### Features

* **scripts:** add dual-language quality gate script ([07e6085](https://github.com/nth-bailey/PolyXML/commit/07e6085b8e1fc9591005fbb832cf189c71d45af3))

# [0.7.0](https://github.com/nth-bailey/PolyXML/compare/v0.6.0...v0.7.0) (2026-09-12)


### Features

* **core:** support mixed-content wildcard fields in deserializer and serializer ([529457f](https://github.com/nth-bailey/PolyXML/commit/529457f2793269d51cd7acd68620b296fb039f5e))

# [0.6.0](https://github.com/nth-bailey/PolyXML/compare/v0.5.1...v0.6.0) (2026-09-12)


### Features

* **python:** universal XML datatype support and class-tagging for binary MessagePack serialization ([b50e62d](https://github.com/nth-bailey/PolyXML/commit/b50e62d0fabe13bb6761c2666d220251c030eec5))

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
