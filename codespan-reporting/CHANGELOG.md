# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.5] - 2020-06-24

### Changed

-   Sections of source code that are marked with primary labels are now rendered
    using the primary highlight color.

## [0.9.4] - 2020-05-18

### Changed

-   We have made the caret rendering easier to read when there are multiple
    labels on the same line. We also avoid printing trailing borders on the
    final source source snippet if no notes are present. Instead of this:
    ```
       ┌─ one_line.rs:3:5
       │
     3 │     v.push(v.pop().unwrap());
       │     - first borrow later used by call
       │       ---- first mutable borrow occurs here
       │            ^ second mutable borrow occurs here
       │
    ```
    …we now render the following:
    ```
       ┌─ one_line.rs:3:5
       │
     3 │     v.push(v.pop().unwrap());
       │     - ---- ^ second mutable borrow occurs here
       │     │ │
       │     │ first mutable borrow occurs here
       │     first borrow later used by call
    ```

### Fixed

-   Diagnostic rendering no longer panics if label ranges are between UTF-8
    character boundaries.

## [0.9.3] - 2020-04-29

### Changed

-   Some panics were fixed when invalid unicode boundaries are supplied.
-   Labels that marked the same span were originally rendered in reverse order.
    This was a mistake! We've now fixed this.

    For example, this diagnostic:
    ```
       ┌─ same_range:1:7
       │
     1 │ ::S { }
       │     - Expected '('
       │     ^ Unexpected '{'
       │
    ```
    …will now be rendered as:
    ```
       ┌─ same_range:1:7
       │
     1 │ ::S { }
       │     ^ Unexpected '{'
       │     - Expected '('
       │
    ```

-   We've reduced the prominence of the 'locus' on source snippets by
    simplifying the border and reducing the spacing around it. This is to help
    focus attention on the underlined source snippet and error messages, rather
    than the location, which should be a secondary focus.

    For example we originally rendered this:
    ```
    error: unknown builtin: `NATRAL`

       ┌── Data/Nat.fun:7:13 ───
       │
     7 │ {-# BUILTIN NATRAL Nat #-}
       │             ^^^^^^ unknown builtin
       │
       = there is a builtin with a similar name: `NATURAL`

    ```
    …and now we render this:
    ```
    error: unknown builtin: `NATRAL`
      ┌─ Data/Nat.fun:7:13
      │
    7 │ {-# BUILTIN NATRAL Nat #-}
      │             ^^^^^^ unknown builtin
      │
      = there is a builtin with a similar name: `NATURAL`

    ```

## [0.9.2] - 2020-03-29

### Changed

-   Render overlapping multiline marks on the same lines of source code.

    For example:
    ```
    error[E0308]: match arms have incompatible types

       ┌── codespan/src/file.rs:1:9 ───
       │
     1 │ ╭         match line_index.compare(self.last_line_index()) {
     2 │ │             Ordering::Less => Ok(self.line_starts()[line_index.to_usize()]),
     3 │ │             Ordering::Equal => Ok(self.source_span().end()),
     4 │ │             Ordering::Greater => LineIndexOutOfBoundsError {
     5 │ │                 given: line_index,
     6 │ │                 max: self.last_line_index(),
     7 │ │             },
     8 │ │         }
       │ ╰─────────' `match` arms have incompatible types
       ·
     2 │               Ordering::Less => Ok(self.line_starts()[line_index.to_usize()]),
       │                                 --------------------------------------------- this is found to be of type `Result<ByteIndex, LineIndexOutOfBoundsError>`
     3 │               Ordering::Equal => Ok(self.source_span().end()),
       │                                  ---------------------------- this is found to be of type `Result<ByteIndex, LineIndexOutOfBoundsError>`
     4 │               Ordering::Greater => LineIndexOutOfBoundsError {
       │ ╭──────────────────────────────────^
     5 │ │                 given: line_index,
     6 │ │                 max: self.last_line_index(),
     7 │ │             },
       │ ╰─────────────^ expected enum `Result`, found struct `LineIndexOutOfBoundsError`
       │
       = expected type `Result<ByteIndex, LineIndexOutOfBoundsError>`
            found type `LineIndexOutOfBoundsError`
    ```
    …is now rendered as:
    ```
    error[E0308]: match arms have incompatible types

       ┌── codespan/src/file.rs:1:9 ───
       │
     1 │   ╭         match line_index.compare(self.last_line_index()) {
     2 │   │             Ordering::Less => Ok(self.line_starts()[line_index.to_usize()]),
       │   │                               --------------------------------------------- this is found to be of type `Result<ByteIndex, LineIndexOutOfBoundsError>`
     3 │   │             Ordering::Equal => Ok(self.source_span().end()),
       │   │                                ---------------------------- this is found to be of type `Result<ByteIndex, LineIndexOutOfBoundsError>`
     4 │   │             Ordering::Greater => LineIndexOutOfBoundsError {
       │ ╭─│──────────────────────────────────^
     5 │ │ │                 given: line_index,
     6 │ │ │                 max: self.last_line_index(),
     7 │ │ │             },
       │ ╰─│─────────────^ expected enum `Result`, found struct `LineIndexOutOfBoundsError`
     8 │   │         }
       │   ╰─────────' `match` arms have incompatible types
       │
       = expected type `Result<ByteIndex, LineIndexOutOfBoundsError>`
            found type `LineIndexOutOfBoundsError`
    ```

## [0.9.1] - 2020-03-23

### Added

-   `codespan_reporting::diagnostic::Diagnostic` now implements `Debug`.

### Changed

-   Single-line labels are now rendered together, under the same source line.

    For example:
    ```
       ┌── one_line.rs:3:5 ───
       │
     3 │     v.push(v.pop().unwrap());
       │     - first borrow later used by call
       ·
     3 │     v.push(v.pop().unwrap());
       │       ---- first mutable borrow occurs here
       ·
     3 │     v.push(v.pop().unwrap());
       │            ^ second mutable borrow occurs here
       │
    ```
    …is now rendered as:
    ```
       ┌── one_line.rs:3:5 ───
       │
     3 │     v.push(v.pop().unwrap());
       │     - first borrow later used by call
       │       ---- first mutable borrow occurs here
       │            ^ second mutable borrow occurs here
       │
    ```

## [0.9.0] - 2020-03-11

### Added

-   The `codespan_reporting::files` module was added as a way to decouple
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

[Unreleased]: https://github.com/brendanzab/codespan/compare/v0.9.5...HEAD
[0.9.4]: https://github.com/brendanzab/codespan/compare/v0.9.4...v0.9.5
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
