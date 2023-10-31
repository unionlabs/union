//! Source code generator.
//!
//! The `srcgen` module contains generic helper routines and classes for
//! generating source code.

#![macro_use]

use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path;

use crate::error;

static SHIFTWIDTH: usize = 4;

/// A macro that simplifies the usage of the Formatter by allowing format
/// strings.
macro_rules! fmtln {
    ($fmt:ident, $fmtstring:expr, $($fmtargs:expr),*) => {
        $fmt.line(format!($fmtstring, $($fmtargs),*))
    };

    ($fmt:ident, $arg:expr) => {
        $fmt.line($arg)
    };

    ($_:tt, $($args:expr),+) => {
        compile_error!("This macro requires at least two arguments: the Formatter instance and a format string.")
    };

    ($_:tt) => {
        compile_error!("This macro requires at least two arguments: the Formatter instance and a format string.")
    };
}

pub(crate) struct Formatter {
    indent: usize,
    lines: Vec<String>,
}

impl Formatter {
    /// Source code formatter class. Used to collect source code to be written
    /// to a file, and keep track of indentation.
    pub fn new() -> Self {
        Self {
            indent: 0,
            lines: Vec::new(),
        }
    }

    /// Increase current indentation level by one.
    pub fn indent_push(&mut self) {
        self.indent += 1;
    }

    /// Decrease indentation by one level.
    pub fn indent_pop(&mut self) {
        assert!(self.indent > 0, "Already at top level indentation");
        self.indent -= 1;
    }

    pub fn indent<T, F: FnOnce(&mut Formatter) -> T>(&mut self, f: F) -> T {
        self.indent_push();
        let ret = f(self);
        self.indent_pop();
        ret
    }

    /// Get the current whitespace indentation in the form of a String.
    fn get_indent(&self) -> String {
        if self.indent == 0 {
            String::new()
        } else {
            format!("{:-1$}", " ", self.indent * SHIFTWIDTH)
        }
    }

    /// Add an indented line.
    pub fn line(&mut self, contents: impl AsRef<str>) {
        let indented_line = format!("{}{}\n", self.get_indent(), contents.as_ref());
        self.lines.push(indented_line);
    }

    /// Pushes an empty line.
    pub fn empty_line(&mut self) {
        self.lines.push("\n".to_string());
    }

    /// Write `self.lines` to a file.
    pub fn update_file(
        &self,
        filename: impl AsRef<str>,
        directory: &str,
    ) -> Result<(), error::Error> {
        #[cfg(target_family = "windows")]
        let path_str = format!("{}\\{}", directory, filename.as_ref());
        #[cfg(not(target_family = "windows"))]
        let path_str = format!("{}/{}", directory, filename.as_ref());

        let path = path::Path::new(&path_str);
        println!("Writing generated file: {}", path.display());
        let mut f = fs::File::create(path)?;

        for l in self.lines.iter().map(|l| l.as_bytes()) {
            f.write_all(l)?;
        }

        Ok(())
    }

    /// Add one or more lines after stripping common indentation.
    pub fn multi_line(&mut self, s: &str) {
        parse_multiline(s).into_iter().for_each(|l| self.line(&l));
    }

    /// Add a comment line.
    pub fn comment(&mut self, s: impl AsRef<str>) {
        fmtln!(self, "// {}", s.as_ref());
    }

    /// Add a (multi-line) documentation comment.
    pub fn doc_comment(&mut self, contents: impl AsRef<str>) {
        parse_multiline(contents.as_ref())
            .iter()
            .map(|l| {
                if l.is_empty() {
                    "///".into()
                } else {
                    format!("/// {}", l)
                }
            })
            .for_each(|s| self.line(s.as_str()));
    }

    /// Add a match expression.
    pub fn add_match(&mut self, m: Match) {
        fmtln!(self, "match {} {{", m.expr);
        self.indent(|fmt| {
            for (&(ref fields, ref body), ref names) in m.arms.iter() {
                // name { fields } | name { fields } => { body }
                let conditions = names
                    .iter()
                    .map(|name| {
                        if !fields.is_empty() {
                            format!("{} {{ {} }}", name, fields.join(", "))
                        } else {
                            name.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" |\n")
                    + " => {";

                fmt.multi_line(&conditions);
                fmt.indent(|fmt| {
                    fmt.line(body);
                });
                fmt.line("}");
            }

            // Make sure to include the catch all clause last.
            if let Some(body) = m.catch_all {
                fmt.line("_ => {");
                fmt.indent(|fmt| {
                    fmt.line(body);
                });
                fmt.line("}");
            }
        });
        self.line("}");
    }
}

/// Compute the indentation of s, or None of an empty line.
fn _indent(s: &str) -> Option<usize> {
    if s.is_empty() {
        None
    } else {
        let t = s.trim_start();
        Some(s.len() - t.len())
    }
}

/// Given a multi-line string, split it into a sequence of lines after
/// stripping a common indentation. This is useful for strings defined with
/// doc strings.
fn parse_multiline(s: &str) -> Vec<String> {
    // Convert tabs into spaces.
    let expanded_tab = format!("{:-1$}", " ", SHIFTWIDTH);
    let lines: Vec<String> = s.lines().map(|l| l.replace("\t", &expanded_tab)).collect();

    // Determine minimum indentation, ignoring the first line and empty lines.
    let indent = lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min();

    // Strip off leading blank lines.
    let mut lines_iter = lines.iter().skip_while(|l| l.is_empty());
    let mut trimmed = Vec::with_capacity(lines.len());

    // Remove indentation (first line is special)
    if let Some(s) = lines_iter.next().map(|l| l.trim()).map(|l| l.to_string()) {
        trimmed.push(s);
    }

    // Remove trailing whitespace from other lines.
    let mut other_lines = if let Some(indent) = indent {
        // Note that empty lines may have fewer than `indent` chars.
        lines_iter
            .map(|l| &l[cmp::min(indent, l.len())..])
            .map(|l| l.trim_end())
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
    } else {
        lines_iter
            .map(|l| l.trim_end())
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
    };

    trimmed.append(&mut other_lines);

    // Strip off trailing blank lines.
    while let Some(s) = trimmed.pop() {
        if s.is_empty() {
            continue;
        } else {
            trimmed.push(s);
            break;
        }
    }

    trimmed
}

/// Match formatting class.
///
/// Match objects collect all the information needed to emit a Rust `match`
/// expression, automatically deduplicating overlapping identical arms.
///
/// Note that this class is ignorant of Rust types, and considers two fields
/// with the same name to be equivalent. BTreeMap/BTreeSet are used to
/// represent the arms in order to make the order deterministic.
pub(crate) struct Match {
    expr: String,
    arms: BTreeMap<(Vec<String>, String), BTreeSet<String>>,
    /// The clause for the placeholder pattern _.
    catch_all: Option<String>,
}

impl Match {
    /// Create a new match statement on `expr`.
    pub fn new(expr: impl Into<String>) -> Self {
        Self {
            expr: expr.into(),
            arms: BTreeMap::new(),
            catch_all: None,
        }
    }

    fn set_catch_all(&mut self, clause: String) {
        assert!(self.catch_all.is_none());
        self.catch_all = Some(clause);
    }

    /// Add an arm that reads fields to the Match statement.
    pub fn arm<T: Into<String>, S: Into<String>>(&mut self, name: T, fields: Vec<S>, body: T) {
        let name = name.into();
        assert!(
            name != "_",
            "catch all clause can't extract fields, use arm_no_fields instead."
        );

        let body = body.into();
        let fields = fields.into_iter().map(|x| x.into()).collect();
        let match_arm = self
            .arms
            .entry((fields, body))
            .or_insert_with(BTreeSet::new);
        match_arm.insert(name);
    }

    /// Adds an arm that doesn't read anythings from the fields to the Match statement.
    pub fn arm_no_fields(&mut self, name: impl Into<String>, body: impl Into<String>) {
        let body = body.into();

        let name = name.into();
        if name == "_" {
            self.set_catch_all(body);
            return;
        }

        let match_arm = self
            .arms
            .entry((Vec::new(), body))
            .or_insert_with(BTreeSet::new);
        match_arm.insert(name);
    }
}

#[cfg(test)]
mod srcgen_tests {
    use super::parse_multiline;
    use super::Formatter;
    use super::Match;

    fn from_raw_string<S: Into<String>>(s: S) -> Vec<String> {
        s.into()
            .trim()
            .split("\n")
            .into_iter()
            .map(|x| format!("{}\n", x))
            .collect()
    }

    #[test]
    fn adding_arms_works() {
        let mut m = Match::new("x");
        m.arm("Orange", vec!["a", "b"], "some body");
        m.arm("Yellow", vec!["a", "b"], "some body");
        m.arm("Green", vec!["a", "b"], "different body");
        m.arm("Blue", vec!["x", "y"], "some body");
        assert_eq!(m.arms.len(), 3);

        let mut fmt = Formatter::new();
        fmt.add_match(m);

        let expected_lines = from_raw_string(
            r#"
match x {
    Green { a, b } => {
        different body
    }
    Orange { a, b } |
    Yellow { a, b } => {
        some body
    }
    Blue { x, y } => {
        some body
    }
}
        "#,
        );
        assert_eq!(fmt.lines, expected_lines);
    }

    #[test]
    fn match_with_catchall_order() {
        // The catchall placeholder must be placed after other clauses.
        let mut m = Match::new("x");
        m.arm("Orange", vec!["a", "b"], "some body");
        m.arm("Green", vec!["a", "b"], "different body");
        m.arm_no_fields("_", "unreachable!()");
        assert_eq!(m.arms.len(), 2); // catchall is not counted

        let mut fmt = Formatter::new();
        fmt.add_match(m);

        let expected_lines = from_raw_string(
            r#"
match x {
    Green { a, b } => {
        different body
    }
    Orange { a, b } => {
        some body
    }
    _ => {
        unreachable!()
    }
}
        "#,
        );
        assert_eq!(fmt.lines, expected_lines);
    }

    #[test]
    fn parse_multiline_works() {
        let input = "\n    hello\n    world\n";
        let expected = vec!["hello", "world"];
        let output = parse_multiline(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn formatter_basic_example_works() {
        let mut fmt = Formatter::new();
        fmt.line("Hello line 1");
        fmt.indent_push();
        fmt.comment("Nested comment");
        fmt.indent_pop();
        fmt.line("Back home again");
        let expected_lines = vec![
            "Hello line 1\n",
            "    // Nested comment\n",
            "Back home again\n",
        ];
        assert_eq!(fmt.lines, expected_lines);
    }

    #[test]
    fn get_indent_works() {
        let mut fmt = Formatter::new();
        let expected_results = vec!["", "    ", "        ", ""];

        let actual_results = Vec::with_capacity(4);
        (0..3).for_each(|_| {
            fmt.get_indent();
            fmt.indent_push();
        });
        (0..3).for_each(|_| fmt.indent_pop());
        fmt.get_indent();

        actual_results
            .into_iter()
            .zip(expected_results.into_iter())
            .for_each(|(actual, expected): (String, &str)| assert_eq!(&actual, expected));
    }

    #[test]
    fn fmt_can_add_type_to_lines() {
        let mut fmt = Formatter::new();
        fmt.line(format!("pub const {}: Type = Type({:#x});", "example", 0,));
        let expected_lines = vec!["pub const example: Type = Type(0x0);\n"];
        assert_eq!(fmt.lines, expected_lines);
    }

    #[test]
    fn fmt_can_add_indented_line() {
        let mut fmt = Formatter::new();
        fmt.line("hello");
        fmt.indent_push();
        fmt.line("world");
        let expected_lines = vec!["hello\n", "    world\n"];
        assert_eq!(fmt.lines, expected_lines);
    }

    #[test]
    fn fmt_can_add_doc_comments() {
        let mut fmt = Formatter::new();
        fmt.doc_comment("documentation\nis\ngood");
        let expected_lines = vec!["/// documentation\n", "/// is\n", "/// good\n"];
        assert_eq!(fmt.lines, expected_lines);
    }

    #[test]
    fn fmt_can_add_doc_comments_with_empty_lines() {
        let mut fmt = Formatter::new();
        fmt.doc_comment(
            r#"documentation
        can be really good.

        If you stick to writing it.
"#,
        );
        let expected_lines = from_raw_string(
            r#"
/// documentation
/// can be really good.
///
/// If you stick to writing it."#,
        );
        assert_eq!(fmt.lines, expected_lines);
    }
}
