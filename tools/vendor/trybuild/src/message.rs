use crate::diff::{Diff, Render};
use crate::error::Error;
use crate::{normalize, term, Expected, Test};
use std::env;
use std::path::Path;
use std::process::Output;
use termcolor::Color::{self, *};

pub(crate) enum Level {
    Fail,
    Warn,
}

pub(crate) use self::Level::*;

pub(crate) fn prepare_fail(err: Error) {
    if err.already_printed() {
        return;
    }

    term::bold_color(Red);
    print!("ERROR");
    term::reset();
    println!(": {}", err);
    println!();
}

pub(crate) fn test_fail(err: Error) {
    if err.already_printed() {
        return;
    }

    term::bold_color(Red);
    println!("error");
    term::color(Red);
    println!("{}", err);
    term::reset();
    println!();
}

pub(crate) fn no_tests_enabled() {
    term::color(Yellow);
    println!("There are no trybuild tests enabled yet.");
    term::reset();
}

pub(crate) fn ok() {
    term::color(Green);
    println!("ok");
    term::reset();
}

pub(crate) fn begin_test(test: &Test, show_expected: bool) {
    let display_name = test.path.as_os_str().to_string_lossy();

    print!("test ");
    term::bold();
    print!("{}", display_name);
    term::reset();

    if show_expected {
        match test.expected {
            Expected::Pass => print!(" [should pass]"),
            Expected::CompileFail => print!(" [should fail to compile]"),
        }
    }

    print!(" ... ");
}

pub(crate) fn failed_to_build(stderr: &str) {
    term::bold_color(Red);
    println!("error");
    snippet(Red, stderr);
    println!();
}

pub(crate) fn should_not_have_compiled() {
    term::bold_color(Red);
    println!("error");
    term::color(Red);
    println!("Expected test case to fail to compile, but it succeeded.");
    term::reset();
    println!();
}

pub(crate) fn write_stderr_wip(wip_path: &Path, stderr_path: &Path, stderr: &str) {
    let wip_path = wip_path.to_string_lossy();
    let stderr_path = stderr_path.to_string_lossy();

    term::bold_color(Yellow);
    println!("wip");
    println!();
    print!("NOTE");
    term::reset();
    println!(": writing the following output to `{}`.", wip_path);
    println!(
        "Move this file to `{}` to accept it as correct.",
        stderr_path,
    );
    snippet(Yellow, stderr);
    println!();
}

pub(crate) fn overwrite_stderr(stderr_path: &Path, stderr: &str) {
    let stderr_path = stderr_path.to_string_lossy();

    term::bold_color(Yellow);
    println!("wip");
    println!();
    print!("NOTE");
    term::reset();
    println!(": writing the following output to `{}`.", stderr_path);
    snippet(Yellow, stderr);
    println!();
}

pub(crate) fn mismatch(expected: &str, actual: &str) {
    term::bold_color(Red);
    println!("mismatch");
    term::reset();
    println!();
    let diff = if env::var_os("TERM").map_or(true, |term| term == "dumb") {
        // No diff in dumb terminal or when TERM is unset.
        None
    } else {
        Diff::compute(expected, actual)
    };
    term::bold_color(Blue);
    println!("EXPECTED:");
    snippet_diff(Blue, expected, diff.as_ref());
    println!();
    term::bold_color(Red);
    println!("ACTUAL OUTPUT:");
    snippet_diff(Red, actual, diff.as_ref());
    print!("note: If the ");
    term::color(Red);
    print!("actual output");
    term::reset();
    println!(" is the correct output you can bless it by rerunning");
    println!("      your test with the environment variable TRYBUILD=overwrite");
    println!();
}

pub(crate) fn output(warnings: &str, output: &Output) {
    let success = output.status.success();
    let stdout = normalize::trim(&output.stdout);
    let stderr = normalize::trim(&output.stderr);
    let has_output = !stdout.is_empty() || !stderr.is_empty();

    if success {
        ok();
        if has_output || !warnings.is_empty() {
            println!();
        }
    } else {
        term::bold_color(Red);
        println!("error");
        term::color(Red);
        if has_output {
            println!("Test case failed at runtime.");
        } else {
            println!("Execution of the test case was unsuccessful but there was no output.");
        }
        term::reset();
        println!();
    }

    self::warnings(warnings);

    let color = if success { Yellow } else { Red };

    for (name, content) in &[("STDOUT", stdout), ("STDERR", stderr)] {
        if !content.is_empty() {
            term::bold_color(color);
            println!("{}:", name);
            snippet(color, &normalize::trim(content));
            println!();
        }
    }
}

pub(crate) fn fail_output(level: Level, stdout: &str) {
    let color = match level {
        Fail => Red,
        Warn => Yellow,
    };

    if !stdout.is_empty() {
        term::bold_color(color);
        println!("STDOUT:");
        snippet(color, &normalize::trim(stdout));
        println!();
    }
}

pub(crate) fn warnings(warnings: &str) {
    if warnings.is_empty() {
        return;
    }

    term::bold_color(Yellow);
    println!("WARNINGS:");
    snippet(Yellow, warnings);
    println!();
}

fn snippet(color: Color, content: &str) {
    snippet_diff(color, content, None);
}

fn snippet_diff(color: Color, content: &str, diff: Option<&Diff>) {
    fn dotted_line() {
        println!("{}", "┈".repeat(60));
    }

    term::color(color);
    dotted_line();

    match diff {
        Some(diff) => {
            for chunk in diff.iter(content) {
                match chunk {
                    Render::Common(s) => {
                        term::color(color);
                        print!("{}", s);
                    }
                    Render::Unique(s) => {
                        term::bold_color(color);
                        print!("\x1B[7m{}", s);
                    }
                }
            }
        }
        None => print!("{}", content),
    }

    term::color(color);
    dotted_line();
    term::reset();
}
