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
            "#,
        ),
    );

    let fizz_buzz_file_id = files.add(
        "FizzBuzz.fun",
        unindent::unindent(
            r#"
                module FizzBuzz where

                fizz₁ : Nat → String
                fizz₁ num = case (mod num 5) (mod num 3) of
                    0 0 => "FizzBuzz"
                    0 _ => "Fizz"
                    _ 0 => "Buzz"
                    _ _ => num

                fizz₂ num =
                    case (mod num 5) (mod num 3) of
                        0 0 => "FizzBuzz"
                        0 _ => "Fizz"
                        _ 0 => "Buzz"
                        _ _ => num
            "#,
        ),
    );

    let diagnostics = [
        // Unknown builtin error
        Diagnostic::new_error(
            "unknown builtin: `NATRAL`",
            Label::new(nat_file_id, 96..102, "unknown builtin"),
        )
        // .with_notes(vec!["there is a builtin with a similar name: `NATURAL`".to_owned()])
        .with_secondary_labels(vec![Label::new(
            nat_file_id,
            96..102,
            "perhaps you meant: `NATURAL`",
        )]),
        // Unused parameter warning
        Diagnostic::new_warning(
            "unused parameter pattern: `n₂`",
            Label::new(nat_file_id, 285..289, "unused parameter"),
        ),
        // .with_notes(vec!["consider using a wildcard pattern: `_`".to_owned()])
        // Unexpected type error
        Diagnostic::new_error(
            "unexpected type in application of `_+_`",
            Label::new(test_file_id, 37..44, "expected `Nat`, found `String`"),
        )
        .with_code("E0001")
        .with_secondary_labels(vec![Label::new(
            nat_file_id,
            130..155,
            "based on the definition of `_+_`",
        )]),
        // Incompatible match clause error
        Diagnostic::new_error(
            "`case` clauses have incompatible types",
            Label::new(
                fizz_buzz_file_id,
                163..166,
                "expected `String`, found `Nat`",
            ),
        )
        .with_code("E0308")
        // .with_notes(vec![unindent::unindent(
        //     "
        //         expected type `String`
        //            found type `Nat`
        //     ",
        // )])
        .with_secondary_labels(vec![
            Label::new(
                fizz_buzz_file_id,
                62..166,
                "`case` clauses have incompatible types",
            ),
            Label::new(
                fizz_buzz_file_id,
                41..47,
                "expected type `String` found here",
            ),
        ]),
        // Incompatible match clause error
        Diagnostic::new_error(
            "`case` clauses have incompatible types",
            Label::new(
                fizz_buzz_file_id,
                303..306,
                "expected `String`, found `Nat`",
            ),
        )
        .with_code("E0308")
        // .with_notes(vec![unindent::unindent(
        //     "
        //         expected type `String`
        //            found type `Nat`
        //     ",
        // )])
        .with_secondary_labels(vec![
            Label::new(
                fizz_buzz_file_id,
                186..306,
                "`case` clauses have incompatible types",
            ),
            Label::new(
                fizz_buzz_file_id,
                233..243,
                "this is found to be of type `String`",
            ),
            Label::new(
                fizz_buzz_file_id,
                259..265,
                "this is found to be of type `String`",
            ),
            Label::new(
                fizz_buzz_file_id,
                281..287,
                "this is found to be of type `String`",
            ),
        ]),
    ];

    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::Config::default();
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    }
}
