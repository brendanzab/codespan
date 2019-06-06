use structopt::StructOpt;

use codespan::Files;
use codespan_reporting::termcolor::StandardStream;
use codespan_reporting::{emit, ColorArg, Diagnostic, Label};

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub struct Opts {
    /// Configure coloring of output
    #[structopt(
        long = "color",
        parse(try_from_str),
        default_value = "auto",
        raw(possible_values = "ColorArg::VARIANTS", case_insensitive = "true")
    )]
    pub color: ColorArg,
}

fn main() {
    let opts = Opts::from_args();
    let mut files = Files::new();

    let nat_file_id = files.add(
        "Data/Nat.fun",
        unindent::unindent(
            "
                module Data.Nat where

                data Nat : Type where
                    zero : Nat
                    succ : Nat → Nat

                {-# BUILTIN NATRAL Nat #-}

                infixl 6 _+_ _-_

                _+_ : Nat → Nat → Nat
                zero    + n₂ = n₂
                succ n₁ + n₂ = succ (n₁ + n₂)

                _-_ : Nat → Nat → Nat
                n₁      - zero    = n₁
                zero    - succ n₂ = zero
                succ n₁ - succ n₂ = n₁ - n₂
            ",
        ),
    );

    let test_file_id = files.add(
        "Test.fun",
        unindent::unindent(
            r#"
                module Test where

                _ : Nat
                _ = 123 + "hello"

                id : {A : Type} → A → A
                id a = a
            "#,
        ),
    );

    let diagnostics = [
        Diagnostic::new_error(
            "unknown builtin: `NATRAL`",
            Label::new(nat_file_id, 96..102, "unknown builtin"),
        )
        .with_secondary_labels(vec![Label::new(
            nat_file_id,
            96..102,
            "perhaps you meant: `NATURAL`",
        )]),

        Diagnostic::new_warning(
            "unused parameter pattern: `n₂`",
            Label::new(nat_file_id, 285..289, ""),
        )
        .with_secondary_labels(vec![Label::new(
            nat_file_id,
            285..289,
            "consider using a wildcard pattern: `_`",
        )]),

        Diagnostic::new_error(
            "unexpected type in application of `_+_`",
            Label::new(test_file_id, 37..44, "expected `Nat` but found `String`"),
        )
        .with_code("E0001")
        .with_secondary_labels(vec![Label::new(
            nat_file_id,
            130..155,
            "based on the definition of `_+_`",
        )]),

        Diagnostic::new_warning(
            "`id` is never used",
            Label::new(test_file_id, 46..82, "definition is never used"),
        ),
    ];

    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::Config::default();
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    }
}
