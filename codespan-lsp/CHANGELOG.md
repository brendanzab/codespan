# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The minimum supported rustc version is now `1.67.0` (was `1.40.0`).
This is because some dependencies now require this Rust version.

### Changed

-   The `lsp-types` dependency was updated to use a version range: `>=0.84, <0.90`,
    which includes the latest updates in `0.89.0`.

## [0.11.1] - 2021-01-18

### Changed

-   The `lsp-types` dependency was updated to use a version range: `>=0.84, <0.89`,
    which includes the latest updates in `0.85.0`, `0.86.0`, `0.87.0`,
    and `0.88.0`.

## [0.11.0] - 2020-11-30

There is now a [code of conduct](https://github.com/brendanzab/codespan/blob/master/CODE_OF_CONDUCT.md)
and a [contributing guide](https://github.com/brendanzab/codespan/blob/master/CONTRIBUTING.md).

### Changed

-   The error type in `codespan-lsp` is replaced with the error type in the `codespan-reporting` crate.
    The error type is now `codespan_reporting::file::Error`.
-   The `lsp-types` dependency was updated to use the version range: `>=0.84, <0.85`.
    Compatibility was broken as a result of the [clarified numeric types] that this crate depended on.
-   The `character_to_line_offset` function was made private to reduce the chance of future public breakages.

[clarified numeric types]: https://github.com/gluon-lang/lsp-types/pull/186

## [0.10.1] - 2020-08-17

### Changed

-   The `lsp-types` dependency was updated to use a version range: `>=0.70, <0.80`,
    which includes the latest updates in `0.79.0`.

## [0.10.0] - 2020-07-20

### Changed

-   `codespan-lsp` only requires `codespan-reporting`, removing its `codespan` dependency.
-   The `lsp-types` dependency was updated to use a version range: `>=0.70,<0.78`,
    which includes the latest updates in `0.77.0`.

## [0.9.5] - 2020-06-24
## [0.9.4] - 2020-05-18

## [0.9.3] - 2020-04-29

### Changed

-   The `lsp-types` dependency was updated to use a version range: `>=0.70,<0.75`,
    which includes the latest updates in `0.74.0`.

## [0.9.2] - 2020-03-29
## [0.9.1] - 2020-03-23
## [0.9.0] - 2020-03-11

### Changed

-   The `lsp-types` dependency was updated to use a version range: `>=0.70,<0.74`,
    which includes the latest updates in `0.73.0`.

### Removed

-   `codespan_lsp` no longer depends on `codespan_reporting`.
-   `make_lsp_severity` and `make_lsp_diagnostic` were removed.
    It's pretty hard to map the diagnostic structure to LSP diagnostics - we
    recommend implementing this as an application-specific concern.

## [0.8.0] - 2020-02-24
## [0.7.0] - 2020-01-06
## [0.6.0] - 2019-12-18
## [0.5.0] - 2019-10-02
## [0.4.1] - 2019-08-25
## [0.4.0] - 2019-08-22
## [0.3.0] - 2019-05-01
## [0.2.1] - 2019-02-26
## [0.2.0] - 2018-10-11

[Unreleased]: https://github.com/brendanzab/codespan/compare/v0.11.1...HEAD
[0.11.1]: https://github.com/brendanzab/codespan/compare/v0.11.0..v0.11.1
[0.11.0]: https://github.com/brendanzab/codespan/compare/v0.10.1..v0.11.0
[0.10.1]: https://github.com/brendanzab/codespan/compare/v0.10.0..v0.10.1
[0.10.0]: https://github.com/brendanzab/codespan/compare/v0.9.5...v0.10.0
[0.9.5]: https://github.com/brendanzab/codespan/compare/v0.9.4...v0.9.5
[0.9.4]: https://github.com/brendanzab/codespan/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/brendanzab/codespan/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/brendanzab/codespan/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/brendanzab/codespan/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/brendanzab/codespan/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/brendanzab/codespan/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/brendanzab/codespan/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/brendanzab/codespan/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/brendanzab/codespan/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/brendanzab/codespan/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/brendanzab/codespan/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/brendanzab/codespan/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/brendanzab/codespan/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/brendanzab/codespan/releases/tag/v0.2.0
