# Darwin Bazel Package Contract

TCFS macOS packaging is still owned by the existing release scripts and
workflows. The Bazel surface added here gives GloriousFlywheel a finite
downstream target to classify before Darwin RBE dispatch work starts.

## Current Target

- `//build/macos:darwin_package_fixture_contract`
- root alias: `//:darwin_package_fixture_contract`
- rule: `tcfs_macos_pkg` in `build/macos/darwin_pkg.bzl`

The fixture target builds a package from declared fake CLI and FileProvider
artifacts. It exists to validate the Bazel rule, target shape, and script
wiring. It is intentionally named as a fixture contract so it cannot be
mistaken for a production release package.

## Rule Contract

`tcfs_macos_pkg` wraps `scripts/macos-build-pkg.sh` and requires:

- a version string
- a declared macOS CLI tarball label
- a declared FileProvider zip label
- the package postinstall script
- optional installer signing identity

The rule produces one `.pkg` output and passes
`TCFS_PKG_STRUCTURE_SMOKE` as a declared tool. It does not discover release
artifacts, fetch from GitHub releases, notarize, staple, or discover signing
credentials.

## Promotion Boundary

Before GloriousFlywheel should classify a TCFS Darwin target as a candidate,
the next target must use real release artifact labels or a source-built Bazel
artifact chain. Signed, notarized, or stapled claims still require
GloriousFlywheel Darwin signing-custody evidence.

The existing blocked `//:darwin_package` placeholder should stay blocked until
a non-fixture production target exists.
