// Needed to supress a 2021 incompatability warning in the macro generated code
// The non_fmt_panic lint is not yet available on most Rust versions
#![allow(unknown_lints, non_fmt_panics)]

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}

#[test]
fn test_changelog() {
    version_sync::assert_contains_regex!("CHANGELOG.md", r#"## \[{version}\]"#);
}

#[test]
fn test_serde_with_dependency() {
    version_sync::assert_contains_regex!(
        "../serde_with/Cargo.toml",
        r#"^serde_with_macros = .*? version = "={version}""#
    );
}
