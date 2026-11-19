## [v0.0.4](https://github.com/shadowmkj/ses/releases/tag/v0.0.4) - 2026-11-19

### 🚀 Features

- *(optimize)* Generalize diff_lines and TokenInterner for raw bytes - ([cc959cd](https://github.com/shadowmkj/ses/commit/cc959cd4e38b1be44316b8e39c457d4fd01c1a62)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### 🐛 Bug Fixes

- *(diff)* Encapsulate from_parts as pub(crate) and add debug validation - ([1a6692a](https://github.com/shadowmkj/ses/commit/1a6692abc6db0866c4bdef807cb346fcedf1b2aa)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### ⚡️ Performance

- *(optimize)* Optimize token interning performance with rustc-hash ([#10](https://github.com/shadowmkj/ses/pull/10)) - ([2e591ae](https://github.com/shadowmkj/ses/commit/2e591aec7d2403344fc44a1d22865b72ab5614c7)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### 👷 CI

- *(release)* Extract release notes directly from CHANGELOG.md - ([cbde57e](https://github.com/shadowmkj/ses/commit/cbde57e757c0dd8700cb30a19549eaa0d66ece4c)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### ⚙️ Misc

- Update README.md with latest API changes - ([3ed0246](https://github.com/shadowmkj/ses/commit/3ed02465cad542139452ade7d69f805b3f5df116)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Add extensive documentations and examples for public API - ([9c35e3a](https://github.com/shadowmkj/ses/commit/9c35e3ac924c1c852c308c8769c4d3493e5a722a)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Flatten public module structure for iterators - ([eda3657](https://github.com/shadowmkj/ses/commit/eda3657e6011b6348ad738f371e10f7bf5c21aa0)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- *(bench)* Add performance benchmarking suite with criterion - ([7801de0](https://github.com/shadowmkj/ses/commit/7801de0b1fb647c5774e9d121a90fbf0f1d8eaf4)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- *(algorithm)* Use Bitset instead of bool vectors in shift algorithm ([#9](https://github.com/shadowmkj/ses/pull/9)) - ([c75d540](https://github.com/shadowmkj/ses/commit/c75d5406fcb80aec28432f80700a470451ae2179)) (from [`@shadowmkj`](https://github.com/shadowmkj))
## [v0.0.3](https://github.com/shadowmkj/ses/releases/tag/v0.0.3) - 2026-11-18

### 🚀 Features

- *(algorithm)* Implement diff boundary shifting to coalesce hunks - ([b58d28f](https://github.com/shadowmkj/ses/commit/b58d28f16cd0ba3e67b717b7f2353d50f2863807)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- *(api)* Stabilize API ergonomics with reference iteration on Diff and EditScript ([#8](https://github.com/shadowmkj/ses/pull/8)) - ([a140100](https://github.com/shadowmkj/ses/commit/a140100adc804e0cf07163153867f5334da0598e)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### 👷 CI

- Add GitHub Actions workflow for continuous integration - ([a492426](https://github.com/shadowmkj/ses/commit/a492426c90d20e22a42caa4071b9cb6ebebf6b6e)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- *(workflow)* Add GitHub Actions workflow for release automation - ([e2647ec](https://github.com/shadowmkj/ses/commit/e2647ec4566e289aba18df21cc2a3780ca9a731f)) (by Milan Pramod)

### ⚙️ Misc

- *(diff)* Add GNU diff compatibility and fuzz tests - ([8d717dc](https://github.com/shadowmkj/ses/commit/8d717dc2fa68fcf1ee964a978552ca0d6b245327)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- *(algorithm)* Refactor myers diff algorithm  ([#1](https://github.com/shadowmkj/ses/pull/1)) - ([d3de393](https://github.com/shadowmkj/ses/commit/d3de3931b44056018ea02603112fc5d26b6d3975)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Add git-cliff configuration for changelog generation - ([d097c35](https://github.com/shadowmkj/ses/commit/d097c35b6174a71f3dbf330a14a51a0725a4cde6)) (by Milan Pramod)
## [v0.0.2](https://github.com/shadowmkj/ses/releases/tag/v0.0.2) - 2026-11-17

### 🚀 Features

- Add sdiff example demonstrating GNU side-by-side diffing - ([9e590d1](https://github.com/shadowmkj/ses/commit/9e590d1995886f6b47a5ba437ac8c1822fe405d9)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### 🐛 Bug Fixes

- *(algorithm)* Reverse k-loop iteration order and add tests - ([6cdbd67](https://github.com/shadowmkj/ses/commit/6cdbd67755b4bbf0df4d6280abc49c24523b510b)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### ⚙️ Misc

- Add files.rs demonstrating raw byte line diffing with diff_slices - ([3197667](https://github.com/shadowmkj/ses/commit/31976679c0075ed2a8a22c218843c72cb23bded6)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Bump version to 0.0.2 - ([690a20a](https://github.com/shadowmkj/ses/commit/690a20aed102241217ebb770c2a4493bd162c24f)) (from [`@shadowmkj`](https://github.com/shadowmkj))
## [v0.0.1](https://github.com/shadowmkj/ses/releases/tag/v0.0.1) - 2026-11-16

### 🚀 Features

- Core linear-space Myers diff library - ([fb3f46e](https://github.com/shadowmkj/ses/commit/fb3f46e05ab7f806e25f925c27d7c2ba42a2e636)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Add prefix/suffix trimming and line token interner optimizations - ([8294dbc](https://github.com/shadowmkj/ses/commit/8294dbc7d5dd6cc506d06df7e34299a854d8c7ef)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Provide both elements in Change::Equal(&'a T, &'a T) - ([1b6e5cf](https://github.com/shadowmkj/ses/commit/1b6e5cf354784ab3e32388f7dab0d0675e138f6e)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### ⚙️ Misc

- Update Cargo.toml - ([72791f1](https://github.com/shadowmkj/ses/commit/72791f12999279a5a81c142068e9a462b7076674)) (from [`@shadowmkj`](https://github.com/shadowmkj))
- Add comprehensive README and crate metadata for 0.0.1 release - ([850661d](https://github.com/shadowmkj/ses/commit/850661d8e9c97fc4c68b67e354e80d0760af92ec)) (from [`@shadowmkj`](https://github.com/shadowmkj))

### 👥 New Contributors

- [`@shadowmkj`](https://github.com/shadowmkj) made their first contribution

