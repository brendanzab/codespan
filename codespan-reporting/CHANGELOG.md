# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-   The `codespan_reporting::files` was added as a way to decouple
    `codespan_reporting` from `codespan`.
    -   `codespan_reporting::files::Files` allows users to implement custom file
        databases that work with `codespan_reporting`. This should make it
        easier to integrate with libraries like Salsa, and also makes it less
        invasive to use `codespan_reporting` on existing projects.
    -   `codespan_reporting::files::SimpleFile` is a simple implementation of
        `codespan_reporting::files::Files` where only a single file is needed.
    -   `codespan_reporting::files::SimpleFiles` is a simple implementation of
        `codespan_reporting::files::Files` where multiple files are needed.

### Changed

-   The `codespan_reporting::diagnostic` module has been greatly revamped,
    making the builder API format more nicely with rustfmt, and allowing for
    multiple primary labels.
-   The output of `codespan_reporting::term::emit` was improved,
    with the following changes:
    -   labels on consecutive lines no longer render breaks between them
    -   source lines are rendered when there is only one line between labels
    -   the inner gutter of code snippets is now aligned consistently
    -   the outer gutter of consecutive code snippets are now aligned consistently
-   `codespan_reporting::term::emit` now takes writers as a trait object (rather
    than using static dispatch) in order to reduce coda bloat and improve
    compile times.
-   The field names in `codespan_reporting::term::Chars` were tweaked for
    consistency.

### Removed

-   `codespan_reporting` no longer depends on `codespan`.
    Note that `codespan` can _still_ be used with `codespan_reporting`,
    as `codespan::Files` now implements `codespan_reporting::files::Files`.

## [0.8.0] - 2020-02-24
## [0.7.0] - 2020-01-06
## [0.6.0] - 2019-12-18
## [0.5.0] - 2019-10-02
## [0.4.1] - 2019-08-25
## [0.4.0] - 2019-08-22
## [0.3.0] - 2019-05-01
## [0.2.1] - 2019-02-26
## [0.2.0] - 2018-10-11

[Unreleased]: https://github.com/brendanzab/codespan/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/brendanzab/codespan/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/brendanzab/codespan/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/brendanzab/codespan/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/brendanzab/codespan/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/brendanzab/codespan/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/brendanzab/codespan/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/brendanzab/codespan/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/brendanzab/codespan/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/brendanzab/codespan/releases/tag/v0.2.0
