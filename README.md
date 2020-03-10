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

Beautiful diagnostic reporting for text-based programming languages.

![Preview](./codespan-reporting/assets/readme_preview.svg?sanitize=true)

## Running the CLI example

To get an idea of what the colored CLI output looks like,
clone the [repository](https://github.com/brendanzab/codespan)
and run the following shell command:

```sh
cargo run --example=term
cargo run --example=term -- --color never
```

We're still working on improving the output - stay tuned for updates!

## Projects using codespan-reporting

`codespan-reporting` is currently used in the following projects:

- [Arret](https://arret-lang.org)
- [Gleam](https://github.com/lpil/gleam/)
- [Gluon](https://github.com/gluon-lang/gluon)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- [Pikelet](https://github.com/pikelet-lang/pikelet)

## Alternatives to codespan-reporting

There are a number of alternatives to `codespan-reporting`, including:

- [annotate-snippets][annotate-snippets]
- [codemap][codemap]
- [language-reporting][language-reporting] (a fork of codespan)

These are all ultimately inspired by rustc's excellent [error reporting infrastructure][librustc_errors].

[annotate-snippets]: https://crates.io/crates/annotate-snippets
[codemap]: https://crates.io/crates/codemap
[language-reporting]: https://crates.io/crates/language-reporting
[librustc_errors]: https://github.com/rust-lang/rust/tree/master/src/librustc_errors
