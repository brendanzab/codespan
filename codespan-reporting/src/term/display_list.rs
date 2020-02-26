use std::ops::{Range, RangeTo};

use crate::diagnostic::Severity;

/// An section of output that can be rendered to a terminal in a context-free way.
///
/// The following diagram gives an overview of each of the parts of the line output:
///
/// ```text
///                    ┌ outer gutter
///                    │ ┌ left border
///                    │ │ ┌ inner gutter
///                    │ │ │   ┌─────────────────────────── source ─────────────────────────────┐
///                    │ │ │   │                                                                │
///                 ┌────────────────────────────────────────────────────────────────────────────
///       header ── │ error[0001]: oh noes, a cupcake has occurred!
///        empty ── │
/// source start ── │    ┌── test:9:0 ───
/// source break ── │    ·
///  source line ── │  9 │   ╭ Cupcake ipsum dolor. Sit amet marshmallow topping cheesecake
///  source line ── │ 10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
///                 │    │ ╭─│─────────^
/// source break ── │    · │ │
///  source line ── │ 33 │ │ │ Muffin danish chocolate soufflé pastry icing bonbon oat cake.
///  source line ── │ 34 │ │ │ Powder cake jujubes oat cake. Lemon drops tootsie roll marshmallow
///                 │    │ │ ╰─────────────────────────────^ blah blah
/// source break ── │    · │
///  source line ── │ 38 │ │   Brownie lemon drops chocolate jelly-o candy canes. Danish marzipan
///  source line ── │ 39 │ │   jujubes soufflé carrot cake marshmallow tiramisu caramels candy canes.
///                 │    │ │           ^^^^^^^^^^^^^^^^^^ blah blah
///                 │    │ │                               -------------------- blah blah
///  source line ── │ 40 │ │   Fruitcake jelly-o danish toffee. Tootsie roll pastry cheesecake
///  source line ── │ 41 │ │   soufflé marzipan. Chocolate bar oat cake jujubes lollipop pastry
///  source line ── │ 42 │ │   cupcake. Candy canes cupcake toffee gingerbread candy canes muffin
///                 │    │ │                                ^^^^^^^^^^^^^^^^^^ blah blah
///                 │    │ ╰──────────^ blah blah
/// source break ── │    ·
///  source line ── │ 82 │     gingerbread toffee chupa chups chupa chups jelly-o cotton candy.
///                 │    │                 ^^^^^^                         ------- blah blah
/// source break ── │    ·
///  source note ── │    = blah blah
///  source note ── │    = blah blah blah
///                 │      blah blah
///  source note ── │    = blah blah blah
///                 │      blah blah
/// ```
///
/// Filler text from http://www.cupcakeipsum.com
pub enum Entry<'files, 'diagnostic> {
    /// Diagnostic header, with severity, code, and message.
    ///
    /// ```text
    /// error[E0001]: unexpected type in `+` application
    /// ```
    Header {
        /// Optional location focus of the diagnostic.
        locus: Option<Locus>,
        /// The severity of this diagnostic.
        severity: Severity,
        /// Optional error code.
        code: Option<&'diagnostic str>,
        /// A message describing the diagnostic.
        message: &'diagnostic str,
    },
    /// Empty line.
    Empty,
    /// The 'location focus' of a source code snippet.
    ///
    /// This is displayed in a way that other tools can understand, for
    /// example when command+clicking in iTerm.
    ///
    /// ```text
    /// ┌── test:9:0 ───
    /// ```
    SourceStart {
        /// The width of the outer gutter.
        outer_padding: usize,
        /// The locus of the source.
        locus: Locus,
    },
    /// A line of source code.
    ///
    /// ```text
    /// 10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
    ///    │ ╭─│─────────^
    /// ```
    SourceLine {
        /// The width of the outer gutter.
        outer_padding: usize,
        /// The line number to render.
        line_number: usize,
        /// Marks to render for this line, in order from left to right.
        ///
        /// This could be `None` if we are going to mark a space.
        marks: Vec<Option<(MarkSeverity, Mark<'diagnostic>)>>,
        /// The source to render.
        source: &'files str,
    },
    /// An empty source line, for providing additional whitespace to source snippets.
    ///
    /// ```text
    /// │ │ │
    /// ```
    SourceEmpty {
        /// The width of the outer gutter.
        outer_padding: usize,
        /// Left marks to render.
        ///
        /// This could be `None` if we are going to mark a space.
        left_marks: Vec<Option<MarkSeverity>>,
    },
    /// A broken source line, for marking skipped sections of source.
    ///
    /// ```text
    /// · │ │
    /// ```
    SourceBreak {
        /// The width of the outer gutter.
        outer_padding: usize,
        /// Left marks to render.
        ///
        /// This could be `None` if we are going to mark a space.
        left_marks: Vec<Option<MarkSeverity>>,
    },
    // Additional notes.
    //
    // ```text
    // = expected type `Int`
    //      found type `String`
    // ```
    SourceNote {
        /// The width of the outer gutter.
        outer_padding: usize,
        /// A (possibly multi-line) message.
        message: &'diagnostic str,
    },
}

/// The 'location focus' of a source code snippet.
pub struct Locus {
    /// The origin of the locus.
    pub origin: String,
    /// The line number.
    pub line_number: usize,
    /// The column number.
    pub column_number: usize,
}

/// A mark to render.
///
/// Locations are relative to the start of where the source cord is rendered.
pub enum Mark<'diagnostic> {
    /// Single-line mark, with an optional message.
    ///
    /// ```text
    /// ^^^^^^^^^ blah blah
    /// ```
    Single(Range<usize>, &'diagnostic str),
    /// Left top corner for multi-line marks.
    ///
    /// ```text
    /// ╭
    /// ```
    MultiTopLeft,
    /// Multi-line mark top.
    ///
    /// ```text
    /// ╭────────────^
    /// ```
    MultiTop(RangeTo<usize>),
    /// Left vertical marks for multi-line marks.
    ///
    /// ```text
    /// │
    /// ```
    MultiLeft,
    /// Multi-line mark bottom, with an optional message.
    ///
    /// ```text
    /// ╰────────────^ blah blah
    /// ```
    MultiBottom(RangeTo<usize>, &'diagnostic str),
}

pub type MarkSeverity = Option<Severity>;
