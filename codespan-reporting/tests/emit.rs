use codespan::Files;
use codespan_reporting::termcolor::{Buffer, WriteColor};
use codespan_reporting::{emit, Config, Diagnostic, DisplayStyle, Label};

mod fizz_buzz {
    use super::*;

    fn emit_test(writer: &mut impl WriteColor, config: &Config) {
        let mut files = Files::new();

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
            Diagnostic::new_error(
                "`case` clauses have incompatible types",
                Label::new(file_id, 163..166, "expected `String`, found `Nat`"),
            )
            .with_code("E0308")
            .with_notes(vec![unindent::unindent(
                "
                    expected type `String`
                       found type `Nat`
                ",
            )])
            .with_secondary_labels(vec![
                Label::new(file_id, 62..166, "`case` clauses have incompatible types"),
                Label::new(file_id, 41..47, "expected type `String` found here"),
            ]),
            // Incompatible match clause error
            Diagnostic::new_error(
                "`case` clauses have incompatible types",
                Label::new(file_id, 303..306, "expected `String`, found `Nat`"),
            )
            .with_code("E0308")
            .with_notes(vec![unindent::unindent(
                "
                    expected type `String`
                       found type `Nat`
                ",
            )])
            .with_secondary_labels(vec![
                Label::new(file_id, 186..306, "`case` clauses have incompatible types"),
                Label::new(file_id, 233..243, "this is found to be of type `String`"),
                Label::new(file_id, 259..265, "this is found to be of type `String`"),
                Label::new(file_id, 281..287, "this is found to be of type `String`"),
            ]),
        ];

        for diagnostic in &diagnostics {
            emit(writer, config, &files, &diagnostic).unwrap();
        }
    }

    #[test]
    fn rich_no_color() {
        let config = Config {
            display_style: DisplayStyle::Rich,
            ..Config::default()
        };

        let mut buffer = Buffer::no_color();
        emit_test(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("rich_no_color", result);
    }

    #[test]
    fn simple_no_color() {
        let config = Config {
            display_style: DisplayStyle::Short,
            ..Config::default()
        };

        let mut buffer = Buffer::no_color();
        emit_test(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("short_no_color", result);
    }
}

mod tabbed {
    use super::*;

    fn emit_test(writer: &mut impl WriteColor, config: &Config) {
        let mut files = Files::new();

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
            Diagnostic::new_warning(
                "unknown weapon `DogJaw`",
                Label::new(file_id, 29..35, "the weapon"),
            ),
            Diagnostic::new_warning(
                "unknown condition `attack-cooldown`",
                Label::new(file_id, 58..73, "the condition"),
            ),
            Diagnostic::new_warning(
                "unknown field `Foo`",
                Label::new(file_id, 75..78, "the field"),
            ),
        ];

        for diagnostic in &diagnostics {
            emit(writer, config, &files, &diagnostic).unwrap();
        }
    }

    #[test]
    fn tab_width_default_no_color() {
        let config = Config::default();

        let mut buffer = Buffer::no_color();
        emit_test(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("tab_width_default_no_color", result);
    }

    #[test]
    fn tab_width_3_no_color() {
        let config = Config {
            tab_width: 3,
            ..Config::default()
        };

        let mut buffer = Buffer::no_color();
        emit_test(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("tab_width_3_no_color", result);
    }

    #[test]
    fn tab_width_6_no_color() {
        let config = Config {
            tab_width: 6,
            ..Config::default()
        };

        let mut buffer = Buffer::no_color();
        emit_test(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("tab_width_6_no_color", result);
    }
}
