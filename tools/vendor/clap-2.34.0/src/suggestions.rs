// Internal
use crate::{app::App, fmt::Format};

/// Produces a string from a given list of possible values which is similar to
/// the passed in value `v` with a certain confidence.
/// Thus in a list of possible values like ["foo", "bar"], the value "fop" will yield
/// `Some("foo")`, whereas "blark" would yield `None`.
#[cfg(feature = "suggestions")]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_lifetimes))]
pub fn did_you_mean<'a, T: ?Sized, I>(v: &str, possible_values: I) -> Option<&'a str>
where
    T: AsRef<str> + 'a,
    I: IntoIterator<Item = &'a T>,
{
    let mut candidate: Option<(f64, &str)> = None;
    for pv in possible_values {
        let confidence = strsim::jaro_winkler(v, pv.as_ref());
        if confidence > 0.8 && (candidate.is_none() || (candidate.as_ref().unwrap().0 < confidence))
        {
            candidate = Some((confidence, pv.as_ref()));
        }
    }
    match candidate {
        None => None,
        Some((_, candidate)) => Some(candidate),
    }
}

#[cfg(not(feature = "suggestions"))]
pub fn did_you_mean<'a, T: ?Sized, I>(_: &str, _: I) -> Option<&'a str>
where
    T: AsRef<str> + 'a,
    I: IntoIterator<Item = &'a T>,
{
    None
}

/// Returns a suffix that can be empty, or is the standard 'did you mean' phrase
pub fn did_you_mean_flag_suffix<'z, T, I>(
    arg: &str,
    args_rest: &'z [&str],
    longs: I,
    subcommands: &'z [App],
) -> (String, Option<&'z str>)
where
    T: AsRef<str> + 'z,
    I: IntoIterator<Item = &'z T>,
{
    if let Some(candidate) = did_you_mean(arg, longs) {
        let suffix = format!(
            "\n\tDid you mean {}{}?",
            Format::Good("--"),
            Format::Good(candidate)
        );
        return (suffix, Some(candidate));
    }

    subcommands
        .iter()
        .filter_map(|subcommand| {
            let opts = subcommand
                .p
                .flags
                .iter()
                .filter_map(|f| f.s.long)
                .chain(subcommand.p.opts.iter().filter_map(|o| o.s.long));

            let candidate = match did_you_mean(arg, opts) {
                Some(candidate) => candidate,
                None => return None,
            };
            let score = match args_rest.iter().position(|x| *x == subcommand.get_name()) {
                Some(score) => score,
                None => return None,
            };

            let suffix = format!(
                "\n\tDid you mean to put '{}{}' after the subcommand '{}'?",
                Format::Good("--"),
                Format::Good(candidate),
                Format::Good(subcommand.get_name())
            );

            Some((score, (suffix, Some(candidate))))
        })
        .min_by_key(|&(score, _)| score)
        .map(|(_, suggestion)| suggestion)
        .unwrap_or_else(|| (String::new(), None))
}

/// Returns a suffix that can be empty, or is the standard 'did you mean' phrase
pub fn did_you_mean_value_suffix<'z, T, I>(arg: &str, values: I) -> (String, Option<&'z str>)
where
    T: AsRef<str> + 'z,
    I: IntoIterator<Item = &'z T>,
{
    match did_you_mean(arg, values) {
        Some(candidate) => {
            let suffix = format!("\n\tDid you mean '{}'?", Format::Good(candidate));
            (suffix, Some(candidate))
        }
        None => (String::new(), None),
    }
}

#[cfg(all(test, features = "suggestions"))]
mod test {
    use super::*;

    #[test]
    fn possible_values_match() {
        let p_vals = ["test", "possible", "values"];
        assert_eq!(did_you_mean("tst", p_vals.iter()), Some("test"));
    }

    #[test]
    fn possible_values_nomatch() {
        let p_vals = ["test", "possible", "values"];
        assert!(did_you_mean("hahaahahah", p_vals.iter()).is_none());
    }

    #[test]
    fn suffix_long() {
        let p_vals = ["test", "possible", "values"];
        let suffix = "\n\tDid you mean \'--test\'?";
        assert_eq!(
            did_you_mean_flag_suffix("tst", p_vals.iter(), []),
            (suffix, Some("test"))
        );
    }

    #[test]
    fn suffix_enum() {
        let p_vals = ["test", "possible", "values"];
        let suffix = "\n\tDid you mean \'test\'?";
        assert_eq!(
            did_you_mean_value_suffix("tst", p_vals.iter()),
            (suffix, Some("test"))
        );
    }
}
