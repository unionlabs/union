#[cfg(feature = "suggestions")]
use std::cmp::Ordering;

// Internal
use crate::builder::Command;

/// Produces multiple strings from a given list of possible values which are similar
/// to the passed in value `v` within a certain confidence by least confidence.
/// Thus in a list of possible values like ["foo", "bar"], the value "fop" will yield
/// `Some("foo")`, whereas "blark" would yield `None`.
#[cfg(feature = "suggestions")]
pub(crate) fn did_you_mean<T, I>(v: &str, possible_values: I) -> Vec<String>
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    let mut candidates: Vec<(f64, String)> = possible_values
        .into_iter()
        .map(|pv| (strsim::jaro_winkler(v, pv.as_ref()), pv.as_ref().to_owned()))
        .filter(|(confidence, _)| *confidence > 0.8)
        .collect();
    candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    candidates.into_iter().map(|(_, pv)| pv).collect()
}

#[cfg(not(feature = "suggestions"))]
pub(crate) fn did_you_mean<T, I>(_: &str, _: I) -> Vec<String>
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    Vec::new()
}

/// Returns a suffix that can be empty, or is the standard 'did you mean' phrase
pub(crate) fn did_you_mean_flag<'a, 'help, I, T>(
    arg: &str,
    remaining_args: &[&str],
    longs: I,
    subcommands: impl IntoIterator<Item = &'a mut Command<'help>>,
) -> Option<(String, Option<String>)>
where
    'help: 'a,
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    use crate::mkeymap::KeyType;

    match did_you_mean(arg, longs).pop() {
        Some(candidate) => Some((candidate, None)),
        None => subcommands
            .into_iter()
            .filter_map(|subcommand| {
                subcommand._build_self();

                let longs = subcommand.get_keymap().keys().filter_map(|a| {
                    if let KeyType::Long(v) = a {
                        Some(v.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                });

                let subcommand_name = subcommand.get_name();

                let candidate = did_you_mean(arg, longs).pop()?;
                let score = remaining_args.iter().position(|x| *x == subcommand_name)?;
                Some((score, (candidate, Some(subcommand_name.to_string()))))
            })
            .min_by_key(|(x, _)| *x)
            .map(|(_, suggestion)| suggestion),
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
    fn possible_values_match() {
        let p_vals = ["test", "temp"];
        assert_eq!(did_you_mean("te", p_vals.iter()), Some("test"));
    }

    #[test]
    fn possible_values_nomatch() {
        let p_vals = ["test", "possible", "values"];
        assert!(did_you_mean("hahaahahah", p_vals.iter()).is_none());
    }

    #[test]
    fn flag() {
        let p_vals = ["test", "possible", "values"];
        assert_eq!(
            did_you_mean_flag("tst", p_vals.iter(), []),
            Some(("test", None))
        );
    }
}
