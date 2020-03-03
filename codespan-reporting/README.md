# codespan-reporting

[![Continuous integration][actions-badge]][actions-url]
[![Crates.io][crate-badge]][crate-url]
[![Docs.rs][docs-badge]][docs-url]
[![Gitter][gitter-badge]][gitter-lobby]

[actions-badge]: https://img.shields.io/github/workflow/status/brendanzab/codespan/Continuous%20integration
[actions-url]: https://github.com/brendanzab/codespan/actions
[crate-url]: https://crates.io/crates/codespan-reporting
[crate-badge]: https://img.shields.io/crates/v/codespan-reporting.svg
[docs-url]: https://docs.rs/codespan-reporting
[docs-badge]: https://docs.rs/codespan-reporting/badge.svg
[gitter-badge]: https://badges.gitter.im/codespan-rs/codespan.svg
[gitter-lobby]: https://gitter.im/codespan-rs/Lobby

Diagnostic reporting for text-based programming languages.

## Running the CLI example

To get an idea of what the colored CLI output looks like with
`codespan-reporting`, clone the [repository](https://github.com/brendanzab/codespan)
and run the following shell command:

```sh
cargo run --example=term
cargo run --example=term -- --color never
```

You should see something like the following in your terminal:

![screenshot](./assets/screenshot.png)

We're still working on the output - stay tuned for updates!
