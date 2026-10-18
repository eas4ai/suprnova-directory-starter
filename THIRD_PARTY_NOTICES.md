# Distribution provenance and third-party notices

The original directory starter is copyright (c) 2026 Shawn McAllister and
licensed under the [MIT license](LICENSE). The package author and repository
identity are `eas4ai`; the existing Pulsar copyright establishes the full name.

## Pulsar (MIT)

Pulsar is an MIT Suprnova application by Shawn McAllister. This starter uses its
account/framework integration patterns and adapted behavior. The distribution
review on 2026-09-09 found exact source matches in:

- `frontend/src/shims-vue.d.ts`
- `src/actions/example_action.rs`
- `src/config/database.rs`
- `src/middleware/logging.rs`
- `src/migrations/m20240101_000002_create_sessions_table.rs`
- `src/migrations/m20240101_000003_create_remember_tokens_table.rs`

The following notice applies to reused and adapted Pulsar material:

```text
MIT License

Copyright (c) 2026 Shawn McAllister

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Dependencies and assets

`Cargo.lock` records Rust dependency versions and checksums, including the exact
Suprnova Git revision named in `Cargo.toml`. `frontend/bun.lock` records the Vue,
Inertia, Vuetify 0 and build dependency resolution. Dependencies retain their own
licenses; this project's MIT license does not replace them. Preserve dependency
license files when distributing vendored source, binaries or frontend bundles.
The source distribution uses package managers rather than vendoring these libraries.

The reviewed application source includes Vue templates, original semantic CSS,
localization text and Rust modules. No bundled stock photographs, purchased logos,
fonts or proprietary media were present in the tracked application asset inventory.
Operator uploads and configured remote logos are operator-provided material and
are not included in this license grant.

## Behavioral references

The purchased Larafast Directories Laravel application is a behavioral reference
only. Its source, templates and assets are not licensed for redistribution here.
The review compared tracked files over 100 bytes with reference files by SHA-256
(excluding dependency directories, build outputs and Git metadata) and found no
exact matches. This is supporting evidence, not a proof against transformed copies;
source/history review and `docs/recon.md` and `docs/pulsar-assessment.md` establish
the independent Rust/Vue implementation boundary. Suprnova.app informed the choice
of Vue/Inertia/Vuetify 0 composition; its branding is not part of this distribution.

Before adding assets or copying code, record its source and permission here and
preserve any required attribution. Do not copy purchased reference material.
