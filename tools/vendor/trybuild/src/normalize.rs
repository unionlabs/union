#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use crate::directory::Directory;
use crate::run::PathDependency;
use std::path::Path;

#[derive(Copy, Clone)]
pub struct Context<'a> {
    pub krate: &'a str,
    pub source_dir: &'a Directory,
    pub workspace: &'a Directory,
    pub input_file: &'a Path,
    pub target_dir: &'a Directory,
    pub path_dependencies: &'a [PathDependency],
}

pub fn trim<S: AsRef<[u8]>>(output: S) -> String {
    let bytes = output.as_ref();
    let mut normalized = String::from_utf8_lossy(bytes).into_owned();

    let len = normalized.trim_end().len();
    normalized.truncate(len);

    if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}

/// For a given compiler output, produces the set of saved outputs against which
/// the compiler's output would be considered correct. If the test's saved
/// stderr file is identical to any one of these variations, the test will pass.
///
/// This is a set rather than just one normalized output in order to avoid
/// breaking existing tests when introducing new normalization steps. Someone
/// may have saved stderr snapshots with an older version of trybuild, and those
/// tests need to continue to pass with newer versions of trybuild.
///
/// There is one "preferred" variation which is what we print when the stderr
/// file is absent or not a match.
pub fn diagnostics(output: &str, context: Context) -> Variations {
    let output = output.replace("\r\n", "\n");

    let variations = [
        Basic,
        StripCouldNotCompile,
        StripCouldNotCompile2,
        StripForMoreInformation,
        StripForMoreInformation2,
        TrimEnd,
        RustLib,
        TypeDirBackslash,
        WorkspaceLines,
        PathDependencies,
        CargoRegistry,
        ArrowOtherCrate,
        RelativeToDir,
        LinesOutsideInputFile,
    ]
    .iter()
    .map(|normalization| apply(&output, *normalization, context))
    .collect();

    Variations { variations }
}

pub struct Variations {
    variations: Vec<String>,
}

impl Variations {
    pub fn preferred(&self) -> &str {
        self.variations.last().unwrap()
    }

    pub fn any<F: FnMut(&str) -> bool>(&self, mut f: F) -> bool {
        self.variations.iter().any(|stderr| f(stderr))
    }
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum Normalization {
    Basic,
    StripCouldNotCompile,
    StripCouldNotCompile2,
    StripForMoreInformation,
    StripForMoreInformation2,
    TrimEnd,
    RustLib,
    TypeDirBackslash,
    WorkspaceLines,
    PathDependencies,
    CargoRegistry,
    ArrowOtherCrate,
    RelativeToDir,
    LinesOutsideInputFile,
    // New normalization steps are to be inserted here at the end so that any
    // snapshots saved before your normalization change remain passing.
}

use self::Normalization::*;

fn apply(original: &str, normalization: Normalization, context: Context) -> String {
    let mut normalized = String::new();

    let lines: Vec<&str> = original.lines().collect();
    let mut filter = Filter {
        all_lines: &lines,
        normalization,
        context,
        hide_numbers: 0,
    };
    for i in 0..lines.len() {
        if let Some(line) = filter.apply(i) {
            normalized += &line;
            if !normalized.ends_with("\n\n") {
                normalized.push('\n');
            }
        }
    }

    trim(normalized)
}

struct Filter<'a> {
    all_lines: &'a [&'a str],
    normalization: Normalization,
    context: Context<'a>,
    hide_numbers: usize,
}

impl<'a> Filter<'a> {
    fn apply(&mut self, index: usize) -> Option<String> {
        let mut line = self.all_lines[index].to_owned();

        if self.hide_numbers > 0 {
            hide_leading_numbers(&mut line);
            self.hide_numbers -= 1;
        }

        let trim_start = line.trim_start();
        let indent = line.len() - trim_start.len();
        let prefix = if trim_start.starts_with("--> ") {
            Some("--> ")
        } else if trim_start.starts_with("::: ") {
            Some("::: ")
        } else {
            None
        };

        if prefix == Some("--> ") && self.normalization < ArrowOtherCrate {
            if let Some(cut_end) = line.rfind(&['/', '\\'][..]) {
                let cut_start = line.find('>').unwrap() + 2;
                line.replace_range(cut_start..cut_end + 1, "$DIR/");
                return Some(line);
            }
        }

        if let Some(prefix) = prefix {
            line = line.replace('\\', "/");
            let line_lower = line.to_ascii_lowercase();
            let target_dir_pat = self
                .context
                .target_dir
                .to_string_lossy()
                .to_ascii_lowercase()
                .replace('\\', "/");
            let source_dir_pat = self
                .context
                .source_dir
                .to_string_lossy()
                .to_ascii_lowercase()
                .replace('\\', "/");
            let mut other_crate = false;
            if line_lower.find(&target_dir_pat) == Some(indent + 4) {
                let mut offset = indent + 4 + target_dir_pat.len();
                let mut out_dir_crate_name = None;
                while let Some(slash) = line[offset..].find('/') {
                    let component = &line[offset..offset + slash];
                    if component == "out" {
                        if let Some(out_dir_crate_name) = out_dir_crate_name {
                            let replacement = format!("$OUT_DIR[{}]", out_dir_crate_name);
                            line.replace_range(indent + 4..offset + 3, &replacement);
                            other_crate = true;
                            break;
                        }
                    } else if component.len() > 17
                        && component.rfind('-') == Some(component.len() - 17)
                        && component[component.len() - 16..].bytes().all(|b| match b {
                            b'0'..=b'9' | b'a'..=b'f' => true,
                            _ => false,
                        })
                    {
                        out_dir_crate_name = Some(&component[..component.len() - 17]);
                    } else {
                        out_dir_crate_name = None;
                    }
                    offset += slash + 1;
                }
            } else if let Some(i) = line_lower.find(&source_dir_pat) {
                if self.normalization >= RelativeToDir && i == indent + 4 {
                    line.replace_range(i..i + source_dir_pat.len(), "");
                    if self.normalization < LinesOutsideInputFile {
                        return Some(line);
                    }
                    let input_file_pat = self
                        .context
                        .input_file
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .replace('\\', "/");
                    if line_lower[i + source_dir_pat.len()..].starts_with(&input_file_pat) {
                        // Keep line numbers only within the input file (the
                        // path passed to our `fn compile_fail`. All other
                        // source files get line numbers erased below.
                        return Some(line);
                    }
                } else {
                    line.replace_range(i..i + source_dir_pat.len() - 1, "$DIR");
                    if self.normalization < LinesOutsideInputFile {
                        return Some(line);
                    }
                }
                other_crate = true;
            } else {
                let workspace_pat = self
                    .context
                    .workspace
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .replace('\\', "/");
                if let Some(i) = line_lower.find(&workspace_pat) {
                    line.replace_range(i..i + workspace_pat.len() - 1, "$WORKSPACE");
                    other_crate = true;
                }
            }
            if self.normalization >= PathDependencies && !other_crate {
                for path_dep in self.context.path_dependencies {
                    let path_dep_pat = path_dep
                        .normalized_path
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .replace('\\', "/");
                    if let Some(i) = line_lower.find(&path_dep_pat) {
                        let var = format!("${}", path_dep.name.to_uppercase().replace('-', "_"));
                        line.replace_range(i..i + path_dep_pat.len() - 1, &var);
                        other_crate = true;
                        break;
                    }
                }
            }
            if self.normalization >= RustLib && !other_crate {
                if let Some(pos) = line.find("/rustlib/src/rust/src/") {
                    // ::: $RUST/src/libstd/net/ip.rs:83:1
                    line.replace_range(line.find(prefix).unwrap() + 4..pos + 17, "$RUST");
                    other_crate = true;
                } else if let Some(pos) = line.find("/rustlib/src/rust/library/") {
                    // ::: $RUST/std/src/net/ip.rs:83:1
                    line.replace_range(line.find(prefix).unwrap() + 4..pos + 25, "$RUST");
                    other_crate = true;
                }
            }
            if self.normalization >= CargoRegistry && !other_crate {
                if let Some(pos) = line.find("/registry/src/github.com-") {
                    // ::: $CARGO/github.com-1ecc6299db9ec823/serde_json-1.0.64/src/de.rs:2584:8
                    line.replace_range(line.find(prefix).unwrap() + 4..pos + 41, "$CARGO");
                    other_crate = true;
                }
            }
            if other_crate && self.normalization >= WorkspaceLines {
                // Blank out line numbers for this particular error since rustc
                // tends to reach into code from outside of the test case. The
                // test stderr shouldn't need to be updated every time we touch
                // those files.
                hide_trailing_numbers(&mut line);
                self.hide_numbers = 1;
                while let Some(next_line) = self.all_lines.get(index + self.hide_numbers) {
                    match next_line.trim_start().chars().next().unwrap_or_default() {
                        '0'..='9' | '|' | '.' => self.hide_numbers += 1,
                        _ => break,
                    }
                }
            }
            return Some(line);
        }

        if line.starts_with("error: aborting due to ") {
            return None;
        }

        if line == "To learn more, run the command again with --verbose." {
            return None;
        }

        if self.normalization >= StripCouldNotCompile {
            if line.starts_with("error: Could not compile `") {
                return None;
            }
        }

        if self.normalization >= StripCouldNotCompile2 {
            if line.starts_with("error: could not compile `") {
                return None;
            }
        }

        if self.normalization >= StripForMoreInformation {
            if line.starts_with("For more information about this error, try `rustc --explain") {
                return None;
            }
        }

        if self.normalization >= StripForMoreInformation2 {
            if line.starts_with("Some errors have detailed explanations:") {
                return None;
            }
            if line.starts_with("For more information about an error, try `rustc --explain") {
                return None;
            }
        }

        if self.normalization >= TrimEnd {
            line.truncate(line.trim_end().len());
        }

        if self.normalization >= TypeDirBackslash {
            if line
                .trim_start()
                .starts_with("= note: required because it appears within the type")
            {
                line = line.replace('\\', "/");
            }
        }

        line = line.replace(self.context.krate, "$CRATE");
        line = replace_case_insensitive(&line, &self.context.source_dir.to_string_lossy(), "$DIR/");
        line = replace_case_insensitive(
            &line,
            &self.context.workspace.to_string_lossy(),
            "$WORKSPACE/",
        );

        Some(line)
    }
}

// "10 | T: Send,"  ->  "   | T: Send,"
fn hide_leading_numbers(line: &mut String) {
    let n = line.bytes().take_while(u8::is_ascii_digit).count();
    for i in 0..n {
        line.replace_range(i..i + 1, " ");
    }
}

// "main.rs:22:29"  ->  "main.rs"
fn hide_trailing_numbers(line: &mut String) {
    for _ in 0..2 {
        let digits = line.bytes().rev().take_while(u8::is_ascii_digit).count();
        if digits == 0 || !line[..line.len() - digits].ends_with(':') {
            return;
        }
        line.truncate(line.len() - digits - 1);
    }
}

fn replace_case_insensitive(line: &str, pattern: &str, replacement: &str) -> String {
    let line_lower = line.to_ascii_lowercase().replace('\\', "/");
    let pattern_lower = pattern.to_ascii_lowercase().replace('\\', "/");
    let mut replaced = String::with_capacity(line.len());

    let line_lower = line_lower.as_str();
    let mut split = line_lower.split(&pattern_lower);
    let mut pos = 0;
    let mut insert_replacement = false;
    while let Some(keep) = split.next() {
        if insert_replacement {
            replaced.push_str(replacement);
            pos += pattern.len();
        }
        let mut keep = &line[pos..pos + keep.len()];
        if insert_replacement {
            let end_of_maybe_path = keep.find(&[' ', ':'][..]).unwrap_or(keep.len());
            replaced.push_str(&keep[..end_of_maybe_path].replace('\\', "/"));
            pos += end_of_maybe_path;
            keep = &keep[end_of_maybe_path..];
        }
        replaced.push_str(keep);
        pos += keep.len();
        insert_replacement = true;
        if replaced.ends_with(|ch: char| ch.is_ascii_alphanumeric()) {
            if let Some(ch) = line[pos..].chars().next() {
                replaced.push(ch);
                pos += ch.len_utf8();
                split = line_lower[pos..].split(&pattern_lower);
                insert_replacement = false;
            }
        }
    }

    replaced
}
