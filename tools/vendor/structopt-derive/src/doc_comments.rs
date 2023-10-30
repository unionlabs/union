//! The preprocessing we apply to doc comments.
//!
//! structopt works in terms of "paragraphs". Paragraph is a sequence of
//! non-empty adjacent lines, delimited by sequences of blank (whitespace only) lines.

use crate::attrs::Method;
use quote::{format_ident, quote};
use std::iter;

pub fn process_doc_comment(lines: Vec<String>, name: &str, preprocess: bool) -> Vec<Method> {
    // multiline comments (`/** ... */`) may have LFs (`\n`) in them,
    // we need to split so we could handle the lines correctly
    //
    // we also need to remove leading and trailing blank lines
    let mut lines: Vec<&str> = lines
        .iter()
        .skip_while(|s| is_blank(s))
        .flat_map(|s| s.split('\n'))
        .collect();

    while let Some(true) = lines.last().map(|s| is_blank(s)) {
        lines.pop();
    }

    // remove one leading space no matter what
    for line in lines.iter_mut() {
        if line.starts_with(' ') {
            *line = &line[1..];
        }
    }

    if lines.is_empty() {
        return vec![];
    }

    let short_name = format_ident!("{}", name);
    let long_name = format_ident!("long_{}", name);

    if let Some(first_blank) = lines.iter().position(|s| is_blank(s)) {
        let (short, long) = if preprocess {
            let paragraphs = split_paragraphs(&lines);
            let short = paragraphs[0].clone();
            let long = paragraphs.join("\n\n");
            (remove_period(short), long)
        } else {
            let short = lines[..first_blank].join("\n");
            let long = lines.join("\n");
            (short, long)
        };

        vec![
            Method::new(short_name, quote!(#short)),
            Method::new(long_name, quote!(#long)),
        ]
    } else {
        let short = if preprocess {
            let s = merge_lines(&lines);
            remove_period(s)
        } else {
            lines.join("\n")
        };

        vec![Method::new(short_name, quote!(#short))]
    }
}

fn split_paragraphs(lines: &[&str]) -> Vec<String> {
    let mut last_line = 0;
    iter::from_fn(|| {
        let slice = &lines[last_line..];
        let start = slice.iter().position(|s| !is_blank(s)).unwrap_or(0);

        let slice = &slice[start..];
        let len = slice
            .iter()
            .position(|s| is_blank(s))
            .unwrap_or_else(|| slice.len());

        last_line += start + len;

        if len != 0 {
            Some(merge_lines(&slice[..len]))
        } else {
            None
        }
    })
    .collect()
}

fn remove_period(mut s: String) -> String {
    if s.ends_with('.') && !s.ends_with("..") {
        s.pop();
    }
    s
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn merge_lines(lines: &[&str]) -> String {
    lines.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" ")
}
