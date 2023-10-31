//! A set of tools designed for processing user defined contract data,
//! which can potientially have abusive size.

use std::collections::{BTreeSet, HashSet};

pub trait LimitedDisplay {
    /// Returns a string representationof the object, which is shorter than or equal to `max_length`.
    /// Implementations must panic if `max_length` is not reasonably large.
    fn to_string_limited(&self, max_length: usize) -> String;
}

impl<E: Ord + AsRef<str>> LimitedDisplay for BTreeSet<E> {
    fn to_string_limited(&self, max_length: usize) -> String {
        collection_to_string_limited(self.iter(), max_length, "{", "}")
    }
}

impl<E: Ord + AsRef<str>> LimitedDisplay for HashSet<E> {
    fn to_string_limited(&self, max_length: usize) -> String {
        // Iteration order in HashSet is undeterminstic. We sort
        // here to be on the safe side and to simplify testing.
        let sorted = BTreeSet::from_iter(self);
        sorted.to_string_limited(max_length)
    }
}

impl<E: AsRef<str>> LimitedDisplay for Vec<E> {
    fn to_string_limited(&self, max_length: usize) -> String {
        collection_to_string_limited(self.iter(), max_length, "[", "]")
    }
}

/// Iterates over a collection and returns a length limited
/// string representation of it, using `opening` and `closing`
/// to surround the collection's content.
fn collection_to_string_limited<E: AsRef<str>, I: ExactSizeIterator<Item = E>>(
    iter: I,
    max_length: usize,
    opening: &str,
    closing: &str,
) -> String {
    let elements_count = iter.len();
    let mut out = String::with_capacity(max_length * 130 / 100);

    let mut first = true;
    out.push_str(opening);
    let mut lengths_stack = Vec::<usize>::new();
    for element in iter {
        lengths_stack.push(out.len());

        if first {
            out.push('"');
            first = false;
        } else {
            out.push_str(", \"");
        }
        out.push_str(element.as_ref());
        out.push('"');

        if out.len() > max_length {
            break;
        };
    }

    if out.len() + closing.len() <= max_length {
        out.push_str(closing);
        out
    } else {
        loop {
            let previous_length = lengths_stack
                .pop()
                .expect("Cannot remove hide enough elements to fit in length limit.");
            let skipped = elements_count - lengths_stack.len();
            let remaining = elements_count - skipped;
            let skipped_text = if remaining == 0 {
                format!("... {} elements", skipped)
            } else {
                format!(", ... {} more", skipped)
            };
            if previous_length + skipped_text.len() + closing.len() <= max_length {
                out.truncate(previous_length);
                out.push_str(&skipped_text);
                out.push_str(closing);
                return out;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works_for_btreeset() {
        let set = BTreeSet::<String>::new();
        assert_eq!(set.to_string_limited(100), "{}");
        assert_eq!(set.to_string_limited(20), "{}");
        assert_eq!(set.to_string_limited(2), "{}");

        let fruits: BTreeSet<String> = [
            "watermelon".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            fruits.to_string_limited(100),
            "{\"apple\", \"banana\", \"watermelon\"}"
        );
        assert_eq!(
            fruits.to_string_limited(33),
            "{\"apple\", \"banana\", \"watermelon\"}"
        );
        assert_eq!(
            fruits.to_string_limited(32),
            "{\"apple\", \"banana\", ... 1 more}"
        );
        assert_eq!(
            fruits.to_string_limited(31),
            "{\"apple\", \"banana\", ... 1 more}"
        );
        assert_eq!(fruits.to_string_limited(30), "{\"apple\", ... 2 more}");
        assert_eq!(fruits.to_string_limited(21), "{\"apple\", ... 2 more}");
        assert_eq!(fruits.to_string_limited(20), "{... 3 elements}");
        assert_eq!(fruits.to_string_limited(16), "{... 3 elements}");
    }

    #[test]
    fn works_for_hashset() {
        let set = HashSet::<String>::new();
        assert_eq!(set.to_string_limited(100), "{}");
        assert_eq!(set.to_string_limited(20), "{}");
        assert_eq!(set.to_string_limited(2), "{}");

        let fruits: HashSet<String> = [
            "watermelon".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            fruits.to_string_limited(100),
            "{\"apple\", \"banana\", \"watermelon\"}"
        );
        assert_eq!(
            fruits.to_string_limited(33),
            "{\"apple\", \"banana\", \"watermelon\"}"
        );
        assert_eq!(
            fruits.to_string_limited(32),
            "{\"apple\", \"banana\", ... 1 more}"
        );
        assert_eq!(
            fruits.to_string_limited(31),
            "{\"apple\", \"banana\", ... 1 more}"
        );
        assert_eq!(fruits.to_string_limited(30), "{\"apple\", ... 2 more}");
        assert_eq!(fruits.to_string_limited(21), "{\"apple\", ... 2 more}");
        assert_eq!(fruits.to_string_limited(20), "{... 3 elements}");
        assert_eq!(fruits.to_string_limited(16), "{... 3 elements}");
    }

    #[test]
    #[should_panic(expected = "Cannot remove hide enough elements to fit in length limit.")]
    fn panics_if_limit_is_too_small_empty() {
        let set = HashSet::<String>::new();
        assert_eq!(set.to_string_limited(1), "{}");
    }

    #[test]
    #[should_panic(expected = "Cannot remove hide enough elements to fit in length limit.")]
    fn panics_if_limit_is_too_small_nonempty() {
        let fruits: HashSet<String> = [
            "watermelon".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ]
        .into_iter()
        .collect();
        assert_eq!(fruits.to_string_limited(15), "{... 3 elements}");
    }

    #[test]
    fn works_for_vectors() {
        let list = Vec::<String>::new();
        assert_eq!(list.to_string_limited(100), "[]");
        assert_eq!(list.to_string_limited(20), "[]");
        assert_eq!(list.to_string_limited(2), "[]");

        let fruits = vec![
            "banana".to_string(),
            "apple".to_string(),
            "watermelon".to_string(),
        ];
        assert_eq!(
            fruits.to_string_limited(100),
            "[\"banana\", \"apple\", \"watermelon\"]"
        );
        assert_eq!(
            fruits.to_string_limited(33),
            "[\"banana\", \"apple\", \"watermelon\"]"
        );
        assert_eq!(
            fruits.to_string_limited(32),
            "[\"banana\", \"apple\", ... 1 more]"
        );
        assert_eq!(
            fruits.to_string_limited(31),
            "[\"banana\", \"apple\", ... 1 more]"
        );
        assert_eq!(fruits.to_string_limited(30), "[\"banana\", ... 2 more]");
        assert_eq!(fruits.to_string_limited(22), "[\"banana\", ... 2 more]");
        assert_eq!(fruits.to_string_limited(21), "[... 3 elements]");
        assert_eq!(fruits.to_string_limited(16), "[... 3 elements]");
    }
}
