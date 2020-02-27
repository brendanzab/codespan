use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{SimpleFile, SimpleFiles};
use codespan_reporting::term::{termcolor::Color, Config, DisplayStyle, Styles};

mod support;

use self::support::TestData;

lazy_static::lazy_static! {
    static ref TEST_CONFIG: Config = Config {
        // Always use blue so tests are consistent across platforms
        styles: Styles::with_blue(Color::Blue),
        ..Config::default()
    };
}

macro_rules! test_emit {
    (rich_color) => {
        #[test]
        fn rich_color() {
            let config = Config {
                display_style: DisplayStyle::Rich,
                ..TEST_CONFIG.clone()
            };

            insta::assert_snapshot!("rich_color", TEST_DATA.emit_color(&config));
        }
    };
    (short_color) => {
        #[test]
        fn short_color() {
            let config = Config {
                display_style: DisplayStyle::Short,
                ..TEST_CONFIG.clone()
            };

            insta::assert_snapshot!("short_color", TEST_DATA.emit_color(&config));
        }
    };
    (rich_no_color) => {
        #[test]
        fn rich_no_color() {
            let config = Config {
                display_style: DisplayStyle::Rich,
                ..TEST_CONFIG.clone()
            };

            insta::assert_snapshot!("rich_no_color", TEST_DATA.emit_no_color(&config));
        }
    };
    (short_no_color) => {
        #[test]
        fn short_no_color() {
            let config = Config {
                display_style: DisplayStyle::Short,
                ..TEST_CONFIG.clone()
            };

            insta::assert_snapshot!("short_no_color", TEST_DATA.emit_no_color(&config));
        }
    };
}

mod empty {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFiles<&'static str, &'static str>> = {
            let files = SimpleFiles::new();

            let diagnostics = vec![
                Diagnostic::bug(),
                Diagnostic::error(),
                Diagnostic::warning(),
                Diagnostic::note(),
                Diagnostic::help(),
                Diagnostic::bug(),
            ];

            TestData { files, diagnostics }
        };
    }

    test_emit!(rich_color);
    test_emit!(short_color);
    test_emit!(rich_no_color);
    test_emit!(short_no_color);
}

mod message {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFiles<&'static str, &'static str>> = {
            let files = SimpleFiles::new();

            let diagnostics = vec![
                Diagnostic::error().with_message("a message"),
                Diagnostic::warning().with_message("a message"),
                Diagnostic::note().with_message("a message"),
                Diagnostic::help().with_message("a message"),
            ];

            TestData { files, diagnostics }
        };
    }

    test_emit!(rich_color);
    test_emit!(short_color);
    test_emit!(rich_no_color);
    test_emit!(short_no_color);
}

mod message_and_notes {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFiles<&'static str, &'static str>> = {
            let files = SimpleFiles::new();

            let diagnostics = vec![
                Diagnostic::error().with_message("a message").with_notes(vec!["a note".to_owned()]),
                Diagnostic::warning().with_message("a message").with_notes(vec!["a note".to_owned()]),
                Diagnostic::note().with_message("a message").with_notes(vec!["a note".to_owned()]),
                Diagnostic::help().with_message("a message").with_notes(vec!["a note".to_owned()]),
            ];

            TestData { files, diagnostics }
        };
    }

    test_emit!(rich_color);
    test_emit!(short_color);
    test_emit!(rich_no_color);
    test_emit!(short_no_color);
}

mod empty_spans {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFile<&'static str, &'static str>> = {
            let file = SimpleFile::new("hello", "Hello world!\nBye world!");
            let eof = file.source().len();

            let diagnostics = vec![
                Diagnostic::note()
                    .with_message("middle")
                    .with_labels(vec![Label::primary((), 6..6).with_message("middle")]),
                Diagnostic::note()
                    .with_message("end of line")
                    .with_labels(vec![Label::primary((), 12..12).with_message("end of line")]),
                Diagnostic::note()
                    .with_message("end of file")
                    .with_labels(vec![Label::primary((), eof..eof).with_message("end of file")]),
            ];

            TestData { files: file, diagnostics }
        };
    }

    test_emit!(rich_color);
    test_emit!(short_color);
    test_emit!(rich_no_color);
    test_emit!(short_no_color);
}

mod multifile {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFiles<&'static str, String>> = {
            let mut files = SimpleFiles::new();

            let file_id1 = files.add(
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

            let file_id2 = files.add(
                "Test.fun",
                unindent::unindent(
                    r#"
                        module Test where

                        _ : Nat
                        _ = 123 + "hello"
                    "#,
                ),
            );

            let diagnostics = vec![
                // Unknown builtin error
                Diagnostic::error()
                    .with_message("unknown builtin: `NATRAL`")
                    .with_labels(vec![Label::primary(file_id1, 96..102).with_message("unknown builtin")])
                    .with_notes(vec![
                        "there is a builtin with a similar name: `NATURAL`".to_owned(),
                    ]),
                // Unused parameter warning
                Diagnostic::warning()
                    .with_message("unused parameter pattern: `n₂`")
                    .with_labels(vec![Label::primary(file_id1, 285..289).with_message("unused parameter")])
                    .with_notes(vec!["consider using a wildcard pattern: `_`".to_owned()]),
                // Unexpected type error
                Diagnostic::error()
                    .with_message("unexpected type in application of `_+_`")
                    .with_code("E0001")
                    .with_labels(vec![
                        Label::primary(file_id2, 37..44).with_message("expected `Nat`, found `String`"),
                        Label::secondary(file_id1, 130..155).with_message("based on the definition of `_+_`"),
                    ]),
            ];

            TestData { files, diagnostics }
        };
    }

    test_emit!(rich_color);
    test_emit!(short_color);
    test_emit!(rich_no_color);
    test_emit!(short_no_color);
}

mod fizz_buzz {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFiles<&'static str, String>> = {
            let mut files = SimpleFiles::new();

            let file_id = files.add(
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

            let diagnostics = vec![
                // Incompatible match clause error
                Diagnostic::error()
                    .with_message("`case` clauses have incompatible types")
                    .with_code("E0308")
                    .with_labels(vec![
                        Label::primary(file_id, 163..166).with_message("expected `String`, found `Nat`"),
                        Label::secondary(file_id, 62..166).with_message("`case` clauses have incompatible types"),
                        Label::secondary(file_id, 41..47).with_message("expected type `String` found here"),
                    ])
                    .with_notes(vec![unindent::unindent(
                        "
                            expected type `String`
                               found type `Nat`
                        ",
                    )]),
                // Incompatible match clause error
                Diagnostic::error()
                    .with_message("`case` clauses have incompatible types")
                    .with_code("E0308")
                    .with_labels(vec![
                        Label::primary(file_id, 303..306).with_message("expected `String`, found `Nat`"),
                        Label::secondary(file_id, 186..306).with_message("`case` clauses have incompatible types"),
                        Label::secondary(file_id, 233..243).with_message("this is found to be of type `String`"),
                        Label::secondary(file_id, 259..265).with_message("this is found to be of type `String`"),
                        Label::secondary(file_id, 281..287).with_message("this is found to be of type `String`"),
                    ])
                    .with_notes(vec![unindent::unindent(
                        "
                            expected type `String`
                               found type `Nat`
                        ",
                    )]),
            ];

            TestData { files, diagnostics }
        };
    }

    test_emit!(rich_color);
    test_emit!(short_color);
    test_emit!(rich_no_color);
    test_emit!(short_no_color);
}

mod tabbed {
    use super::*;

    lazy_static::lazy_static! {
        static ref TEST_DATA: TestData<'static, SimpleFiles<&'static str, String>> = {
            let mut files = SimpleFiles::new();

            let file_id = files.add(
                "tabbed",
                [
                    "Entity:",
                    "\tArmament:",
                    "\t\tWeapon: DogJaw",
                    "\t\tReloadingCondition:\tattack-cooldown",
                    "\tFoo: Bar",
                ]
                .join("\n"),
            );

            let diagnostics = vec![
                Diagnostic::warning()
                    .with_message("unknown weapon `DogJaw`")
                    .with_labels(vec![Label::primary(file_id, 29..35).with_message("the weapon")]),
                Diagnostic::warning()
                    .with_message("unknown condition `attack-cooldown`")
                    .with_labels(vec![Label::primary(file_id, 58..73).with_message("the condition")]),
                Diagnostic::warning()
                    .with_message("unknown field `Foo`")
                    .with_labels(vec![Label::primary(file_id, 75..78).with_message("the field")]),
            ];

            TestData { files, diagnostics }
        };
    }

    #[test]
    fn tab_width_default_no_color() {
        let config = TEST_CONFIG.clone();

        insta::assert_snapshot!(
            "tab_width_default_no_color",
            TEST_DATA.emit_no_color(&config)
        );
    }

    #[test]
    fn tab_width_3_no_color() {
        let config = Config {
            tab_width: 3,
            ..TEST_CONFIG.clone()
        };

        insta::assert_snapshot!("tab_width_3_no_color", TEST_DATA.emit_no_color(&config));
    }

    #[test]
    fn tab_width_6_no_color() {
        let config = Config {
            tab_width: 6,
            ..TEST_CONFIG.clone()
        };

        insta::assert_snapshot!("tab_width_6_no_color", TEST_DATA.emit_no_color(&config));
    }
}
