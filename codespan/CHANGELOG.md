# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.1] - 2021-01-18

### Fixed

-   Removed an erroneous feature gate from the implementation of
    `codespan_reporting::Files` for `codespan::Files`.

## [0.11.0] - 2020-11-30

There is now a [code of conduct](https://github.com/brendanzab/codespan/blob/master/CODE_OF_CONDUCT.md)
and a [contributing guide](https://github.com/brendanzab/codespan/blob/master/CONTRIBUTING.md).

Some versions were skipped to sync up with the `codespan-lsp` crate. The release
process has been changed so this should not happen again.

### Changed

-   This crate now depends on `codespan-reporting` non-optionally
    because of the new error type.

### Fixed

-   Building the crate on operating systems other than unix/windows
    with the `serialization` feature enabled.

## [0.9.5] - 2020-06-24

### Fixed

-   Building the crate with the `reporting` feature disabled.

## [0.9.4] - 2020-05-18
## [0.9.3] - 2020-04-29
## [0.9.2] - 2020-03-29

## [0.9.1] - 2020-03-23

### Added

-   `From<Span>` is now implemented for `Range<usize>` and `Range<RawIndex>`.
-   More `From<_>` conversions are now implemented for index and offset types.

## [0.9.0] - 2020-03-11

### Added

-   `codespan` now depends on `codespan_reporting`. This can be disabled via the `reporting` feature.
-   `codespan::Files` now implements `codespan_reporting::files::Files`

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
[0.11.0]: https://github.com/brendanzab/codespan/compare/v0.9.5...v0.11.0
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
