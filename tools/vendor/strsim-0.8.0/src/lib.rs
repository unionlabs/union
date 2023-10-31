//! This library implements string similarity metrics.

use std::char;
use std::cmp::{max, min};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum StrSimError {
    DifferentLengthArgs
}

pub type HammingResult = Result<usize, StrSimError>;

/// Calculates the number of positions in the two strings where the characters
/// differ. Returns an error if the strings have different lengths.
///
/// ```
/// use strsim::hamming;
///
/// match hamming("hamming", "hammers") {
///     Ok(distance) => assert_eq!(3, distance),
///     Err(why) => panic!("{:?}", why)
/// }
/// ```
pub fn hamming(a: &str, b: &str) -> HammingResult {
    let (mut ita, mut itb, mut count) = (a.chars(), b.chars(), 0);
    loop {
        match (ita.next(), itb.next()){
            (Some(x), Some(y)) => if x != y { count += 1 },
            (None, None) => return Ok(count),
            _ => return Err(StrSimError::DifferentLengthArgs),
        }
    }
}

/// Calculates the Jaro similarity between two strings. The returned value
/// is between 0.0 and 1.0 (higher value means more similar).
///
/// ```
/// use strsim::jaro;
///
/// assert!((0.392 - jaro("Friedrich Nietzsche", "Jean-Paul Sartre")).abs() <
///         0.001);
/// ```
pub fn jaro(a: &str, b: &str) -> f64 {
    if a == b { return 1.0; }

    let a_len = a.chars().count();
    let b_len = b.chars().count();

    // The check for lengths of one here is to prevent integer overflow when
    // calculating the search range.
    if a_len == 0 || b_len == 0 || (a_len == 1 && b_len == 1) {
        return 0.0;
    }

    let search_range = (max(a_len, b_len) / 2) - 1;

    let mut b_consumed = Vec::with_capacity(b_len);
    for _ in 0..b_len {
        b_consumed.push(false);
    }
    let mut matches = 0.0;

    let mut transpositions = 0.0;
    let mut b_match_index = 0;

    for (i, a_char) in a.chars().enumerate() {
        let min_bound =
            // prevent integer wrapping
            if i > search_range {
                max(0, i - search_range)
            } else {
                0
            };

        let max_bound = min(b_len - 1, i + search_range);

        if min_bound > max_bound {
            continue;
        }

        for (j, b_char) in b.chars().enumerate() {
            if min_bound <= j && j <= max_bound && a_char == b_char &&
               !b_consumed[j] {
                b_consumed[j] = true;
                matches += 1.0;

                if j < b_match_index {
                    transpositions += 1.0;
                }
                b_match_index = j;

                break;
            }
        }
    }

    if matches == 0.0 {
        0.0
    } else {
        (1.0 / 3.0) * ((matches / a_len as f64) +
                       (matches / b_len as f64) +
                       ((matches - transpositions) / matches))
    }
}

/// Like Jaro but gives a boost to strings that have a common prefix.
///
/// ```
/// use strsim::jaro_winkler;
///
/// assert!((0.911 - jaro_winkler("cheeseburger", "cheese fries")).abs() <
///         0.001);
/// ```
pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let jaro_distance = jaro(a, b);

    // Don't limit the length of the common prefix
    let prefix_length = a.chars()
                         .zip(b.chars())
                         .take_while(|&(a_char, b_char)| a_char == b_char)
                         .count();

    let jaro_winkler_distance =
        jaro_distance + (0.1 * prefix_length as f64 * (1.0 - jaro_distance));

    if jaro_winkler_distance <= 1.0 {
        jaro_winkler_distance
    } else {
        1.0
    }
}

/// Calculates the minimum number of insertions, deletions, and substitutions
/// required to change one string into the other.
///
/// ```
/// use strsim::levenshtein;
///
/// assert_eq!(3, levenshtein("kitten", "sitting"));
/// ```
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b { return 0; }

    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut cache: Vec<usize> = (1..b_len+1).collect();

    let mut result = 0;
    let mut distance_a;
    let mut distance_b;

    for (i, a_char) in a.chars().enumerate() {
        result = i;
        distance_b = i;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            distance_a = distance_b + cost;
            distance_b = cache[j];
            result = min(result + 1, min(distance_a, distance_b + 1));
            cache[j] = result;
        }
    }

    result
}

/// Calculates a normalized score of the Levenshtein algorithm between 0.0 and
/// 1.0 (inclusive), where 1.0 means the strings are the same.
///
/// ```
/// use strsim::normalized_levenshtein;
///
/// assert!((normalized_levenshtein("kitten", "sitting") - 0.57142).abs() < 0.00001);
/// assert!((normalized_levenshtein("", "") - 1.0).abs() < 0.00001);
/// assert!(normalized_levenshtein("", "second").abs() < 0.00001);
/// assert!(normalized_levenshtein("first", "").abs() < 0.00001);
/// assert!((normalized_levenshtein("string", "string") - 1.0).abs() < 0.00001);
/// ```
pub fn normalized_levenshtein(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    1.0 - (levenshtein(a, b) as f64) / (a.chars().count().max(b.chars().count()) as f64)
}

/// Like Levenshtein but allows for adjacent transpositions. Each substring can
/// only be edited once.
///
/// ```
/// use strsim::osa_distance;
///
/// assert_eq!(3, osa_distance("ab", "bca"));
/// ```
pub fn osa_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    if a == b { return 0; }
    else if a_len == 0 { return b_len; }
    else if b_len == 0 { return a_len; }

    let mut prev_two_distances: Vec<usize> = Vec::with_capacity(b_len + 1);
    let mut prev_distances: Vec<usize> = Vec::with_capacity(b_len + 1);
    let mut curr_distances: Vec<usize> = Vec::with_capacity(b_len + 1);

    let mut prev_a_char = char::MAX;
    let mut prev_b_char = char::MAX;

    for i in 0..(b_len + 1) {
        prev_two_distances.push(i);
        prev_distances.push(i);
        curr_distances.push(0);
    }

    for (i, a_char) in a.chars().enumerate() {
        curr_distances[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            curr_distances[j + 1] = min(curr_distances[j] + 1,
                                        min(prev_distances[j + 1] + 1,
                                            prev_distances[j] + cost));
            if i > 0 && j > 0 && a_char != b_char &&
               a_char == prev_b_char && b_char == prev_a_char {
                curr_distances[j + 1] = min(curr_distances[j + 1],
                                            prev_two_distances[j - 1] + 1);
            }

            prev_b_char = b_char;
        }

        prev_two_distances.clone_from(&prev_distances);
        prev_distances.clone_from(&curr_distances);
        prev_a_char = a_char;
    }

    curr_distances[b_len]

}

/// Like optimal string alignment, but substrings can be edited an unlimited
/// number of times, and the triangle inequality holds.
///
/// ```
/// use strsim::damerau_levenshtein;
///
/// assert_eq!(2, damerau_levenshtein("ab", "bca"));
/// ```
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    if a == b { return 0; }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }

    let mut distances = vec![vec![0; b_len + 2]; a_len + 2];
    let max_distance = a_len + b_len;
    distances[0][0] = max_distance;

    for i in 0..(a_len + 1) {
        distances[i + 1][0] = max_distance;
        distances[i + 1][1] = i;
    }

    for j in 0..(b_len + 1) {
        distances[0][j + 1] = max_distance;
        distances[1][j + 1] = j;
    }

    let mut chars: HashMap<char, usize> = HashMap::new();

    for i in 1..(a_len + 1) {
        let mut db = 0;

        for j in 1..(b_len + 1) {
            let k = match chars.get(&b_chars[j - 1]) {
                Some(value) => value.clone(),
                None => 0
            };

            let l = db;

            let mut cost = 1;
            if a_chars[i - 1] == b_chars[j - 1] {
                cost = 0;
                db = j;
            }

            let substitution_cost = distances[i][j] + cost;
            let insertion_cost = distances[i][j + 1] + 1;
            let deletion_cost = distances[i + 1][j] + 1;
            let transposition_cost = distances[k][l] + (i - k - 1) + 1 +
                                     (j - l - 1);

            distances[i + 1][j + 1] = min(substitution_cost,
                                      min(insertion_cost,
                                      min(deletion_cost,
                                          transposition_cost)));
        }

        chars.insert(a_chars[i - 1], i);
    }

    distances[a_len + 1][b_len + 1]
}

/// Calculates a normalized score of the Damerau–Levenshtein algorithm between
/// 0.0 and 1.0 (inclusive), where 1.0 means the strings are the same.
///
/// ```
/// use strsim::normalized_damerau_levenshtein;
///
/// assert!((normalized_damerau_levenshtein("levenshtein", "löwenbräu") - 0.27272).abs() < 0.00001);
/// assert!((normalized_damerau_levenshtein("", "") - 1.0).abs() < 0.00001);
/// assert!(normalized_damerau_levenshtein("", "flower").abs() < 0.00001);
/// assert!(normalized_damerau_levenshtein("tree", "").abs() < 0.00001);
/// assert!((normalized_damerau_levenshtein("sunglasses", "sunglasses") - 1.0).abs() < 0.00001);
/// ```
pub fn normalized_damerau_levenshtein(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    1.0 - (damerau_levenshtein(a, b) as f64) / (a.chars().count().max(b.chars().count()) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamming_empty() {
        match hamming("", "") {
            Ok(distance) => { assert_eq!(0, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_same() {
        match hamming("hamming", "hamming") {
            Ok(distance) => { assert_eq!(0, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_diff() {
        match hamming("hamming", "hammers") {
            Ok(distance) => { assert_eq!(3, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_diff_multibyte() {
        match hamming("hamming", "h香mmüng") {
            Ok(distance) => { assert_eq!(2, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn hamming_unequal_length() {
        match hamming("ham", "hamming") {
            Ok(_) => { panic!(); },
            Err(why) => { assert_eq!(why, StrSimError::DifferentLengthArgs); }
        }
    }

    #[test]
    fn hamming_names() {
        match hamming("Friedrich Nietzs", "Jean-Paul Sartre") {
            Ok(distance) => { assert_eq!(14, distance); },
            Err(why) => { panic!("{:?}", why); }
        }
    }

    #[test]
    fn jaro_both_empty() {
       assert_eq!(1.0, jaro("", ""));
    }

    #[test]
    fn jaro_first_empty() {
        assert_eq!(0.0, jaro("", "jaro"));
    }

    #[test]
    fn jaro_second_empty() {
        assert_eq!(0.0, jaro("distance", ""));
    }

    #[test]
    fn jaro_same() {
        assert_eq!(1.0, jaro("jaro", "jaro"));
    }

    #[test]
    fn jaro_multibyte() {
        assert!((0.818 - jaro("testabctest", "testöঙ香test")) < 0.001);
        assert!((0.818 - jaro("testöঙ香test", "testabctest")) < 0.001);
    }

    #[test]
    fn jaro_diff_short() {
        assert!((0.767 - jaro("dixon", "dicksonx")).abs() < 0.001);
    }

    #[test]
    fn jaro_diff_one_character() {
        assert_eq!(0.0, jaro("a", "b"));
    }

    #[test]
    fn jaro_diff_one_and_two() {
        assert!((0.83 - jaro("a", "ab")).abs() < 0.01);
    }

    #[test]
    fn jaro_diff_two_and_one() {
        assert!((0.83 - jaro("ab", "a")).abs() < 0.01);
    }

    #[test]
    fn jaro_diff_no_transposition() {
        assert!((0.822 - jaro("dwayne", "duane")).abs() < 0.001);
    }

    #[test]
    fn jaro_diff_with_transposition() {
        assert!((0.944 - jaro("martha", "marhta")).abs() < 0.001);
    }

    #[test]
    fn jaro_names() {
        assert!((0.392 - jaro("Friedrich Nietzsche",
                              "Jean-Paul Sartre")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_both_empty() {
        assert_eq!(1.0, jaro_winkler("", ""));
    }

    #[test]
    fn jaro_winkler_first_empty() {
        assert_eq!(0.0, jaro_winkler("", "jaro-winkler"));
    }

    #[test]
    fn jaro_winkler_second_empty() {
        assert_eq!(0.0, jaro_winkler("distance", ""));
    }

    #[test]
    fn jaro_winkler_same() {
        assert_eq!(1.0, jaro_winkler("Jaro-Winkler", "Jaro-Winkler"));
    }

    #[test]
    fn jaro_winkler_multibyte() {
        assert!((0.89 - jaro_winkler("testabctest", "testöঙ香test")).abs() <
                0.001);
        assert!((0.89 - jaro_winkler("testöঙ香test", "testabctest")).abs() <
                0.001);
    }

    #[test]
    fn jaro_winkler_diff_short() {
        assert!((0.813 - jaro_winkler("dixon", "dicksonx")).abs() < 0.001);
        assert!((0.813 - jaro_winkler("dicksonx", "dixon")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_diff_one_character() {
        assert_eq!(0.0, jaro_winkler("a", "b"));
    }

    #[test]
    fn jaro_winkler_diff_no_transposition() {
        assert!((0.840 - jaro_winkler("dwayne", "duane")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_diff_with_transposition() {
        assert!((0.961 - jaro_winkler("martha", "marhta")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_names() {
        assert!((0.562 - jaro_winkler("Friedrich Nietzsche",
                                      "Fran-Paul Sartre")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_long_prefix() {
        assert!((0.911 - jaro_winkler("cheeseburger", "cheese fries")).abs() <
                0.001);
    }

    #[test]
    fn jaro_winkler_more_names() {
        assert!((0.868 - jaro_winkler("Thorkel", "Thorgier")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_length_of_one() {
        assert!((0.738 - jaro_winkler("Dinsdale", "D")).abs() < 0.001);
    }

    #[test]
    fn jaro_winkler_very_long_prefix() {
        assert!((1.0 - jaro_winkler("thequickbrownfoxjumpedoverx",
                                    "thequickbrownfoxjumpedovery")).abs() <
                0.001);
    }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(0, levenshtein("", ""));
    }

    #[test]
    fn levenshtein_same() {
        assert_eq!(0, levenshtein("levenshtein", "levenshtein"));
    }

    #[test]
    fn levenshtein_diff_short() {
        assert_eq!(3, levenshtein("kitten", "sitting"));
    }

    #[test]
    fn levenshtein_diff_with_space() {
        assert_eq!(5, levenshtein("hello, world", "bye, world"));
    }

    #[test]
    fn levenshtein_diff_multibyte() {
        assert_eq!(3, levenshtein("öঙ香", "abc"));
        assert_eq!(3, levenshtein("abc", "öঙ香"));
    }

    #[test]
    fn levenshtein_diff_longer() {
        let a = "The quick brown fox jumped over the angry dog.";
        let b = "Lorem ipsum dolor sit amet, dicta latine an eam.";
        assert_eq!(37, levenshtein(a, b));
    }

    #[test]
    fn levenshtein_first_empty() {
        assert_eq!(7, levenshtein("", "sitting"));
    }

    #[test]
    fn levenshtein_second_empty() {
        assert_eq!(6, levenshtein("kitten", ""));
    }

    #[test]
    fn normalized_levenshtein_diff_short() {
        assert!((normalized_levenshtein("kitten", "sitting") - 0.57142).abs() < 0.00001);
    }

    #[test]
    fn normalized_levenshtein_for_empty_strings() {
        assert!((normalized_levenshtein("", "") - 1.0).abs() < 0.00001);
    }

    #[test]
    fn normalized_levenshtein_first_empty() {
        assert!(normalized_levenshtein("", "second").abs() < 0.00001);
    }

    #[test]
    fn normalized_levenshtein_second_empty() {
        assert!(normalized_levenshtein("first", "").abs() < 0.00001);
    }

    #[test]
    fn normalized_levenshtein_identical_strings() {
        assert!((normalized_levenshtein("identical", "identical") - 1.0).abs() < 0.00001);
    }

    #[test]
    fn osa_distance_empty() {
        assert_eq!(0, osa_distance("", ""));
    }

    #[test]
    fn osa_distance_same() {
        assert_eq!(0, osa_distance("damerau", "damerau"));
    }

    #[test]
    fn osa_distance_first_empty() {
        assert_eq!(7, osa_distance("", "damerau"));
    }

    #[test]
    fn osa_distance_second_empty() {
        assert_eq!(7, osa_distance("damerau", ""));
    }

    #[test]
    fn osa_distance_diff() {
        assert_eq!(3, osa_distance("ca", "abc"));
    }

    #[test]
    fn osa_distance_diff_short() {
        assert_eq!(3, osa_distance("damerau", "aderua"));
    }

    #[test]
    fn osa_distance_diff_reversed() {
        assert_eq!(3, osa_distance("aderua", "damerau"));
    }

    #[test]
    fn osa_distance_diff_multibyte() {
        assert_eq!(3, osa_distance("öঙ香", "abc"));
        assert_eq!(3, osa_distance("abc", "öঙ香"));
    }

    #[test]
    fn osa_distance_diff_unequal_length() {
        assert_eq!(6, osa_distance("damerau", "aderuaxyz"));
    }

    #[test]
    fn osa_distance_diff_unequal_length_reversed() {
        assert_eq!(6, osa_distance("aderuaxyz", "damerau"));
    }

    #[test]
    fn osa_distance_diff_comedians() {
        assert_eq!(5, osa_distance("Stewart", "Colbert"));
    }

    #[test]
    fn osa_distance_many_transpositions() {
        assert_eq!(4, osa_distance("abcdefghijkl", "bacedfgihjlk"));
    }

    #[test]
    fn osa_distance_diff_longer() {
        let a = "The quick brown fox jumped over the angry dog.";
        let b = "Lehem ipsum dolor sit amet, dicta latine an eam.";
        assert_eq!(36, osa_distance(a, b));
    }

    #[test]
    fn osa_distance_beginning_transposition() {
        assert_eq!(1, osa_distance("foobar", "ofobar"));
    }

    #[test]
    fn osa_distance_end_transposition() {
        assert_eq!(1, osa_distance("specter", "spectre"));
    }

    #[test]
    fn osa_distance_restricted_edit() {
        assert_eq!(4, osa_distance("a cat", "an abct"));
    }

    #[test]
    fn damerau_levenshtein_empty() {
        assert_eq!(0, damerau_levenshtein("", ""));
    }

    #[test]
    fn damerau_levenshtein_same() {
        assert_eq!(0, damerau_levenshtein("damerau", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_first_empty() {
        assert_eq!(7, damerau_levenshtein("", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_second_empty() {
        assert_eq!(7, damerau_levenshtein("damerau", ""));
    }

    #[test]
    fn damerau_levenshtein_diff() {
        assert_eq!(2, damerau_levenshtein("ca", "abc"));
    }

    #[test]
    fn damerau_levenshtein_diff_short() {
        assert_eq!(3, damerau_levenshtein("damerau", "aderua"));
    }

    #[test]
    fn damerau_levenshtein_diff_reversed() {
        assert_eq!(3, damerau_levenshtein("aderua", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_diff_multibyte() {
        assert_eq!(3, damerau_levenshtein("öঙ香", "abc"));
        assert_eq!(3, damerau_levenshtein("abc", "öঙ香"));
    }

    #[test]
    fn damerau_levenshtein_diff_unequal_length() {
        assert_eq!(6, damerau_levenshtein("damerau", "aderuaxyz"));
    }

    #[test]
    fn damerau_levenshtein_diff_unequal_length_reversed() {
        assert_eq!(6, damerau_levenshtein("aderuaxyz", "damerau"));
    }

    #[test]
    fn damerau_levenshtein_diff_comedians() {
        assert_eq!(5, damerau_levenshtein("Stewart", "Colbert"));
    }

    #[test]
    fn damerau_levenshtein_many_transpositions() {
        assert_eq!(4, damerau_levenshtein("abcdefghijkl", "bacedfgihjlk"));
    }

    #[test]
    fn damerau_levenshtein_diff_longer() {
        let a = "The quick brown fox jumped over the angry dog.";
        let b = "Lehem ipsum dolor sit amet, dicta latine an eam.";
        assert_eq!(36, damerau_levenshtein(a, b));
    }

    #[test]
    fn damerau_levenshtein_beginning_transposition() {
        assert_eq!(1, damerau_levenshtein("foobar", "ofobar"));
    }

    #[test]
    fn damerau_levenshtein_end_transposition() {
        assert_eq!(1, damerau_levenshtein("specter", "spectre"));
    }

    #[test]
    fn damerau_levenshtein_unrestricted_edit() {
        assert_eq!(3, damerau_levenshtein("a cat", "an abct"));
    }

    #[test]
    fn normalized_damerau_levenshtein_diff_short() {
        assert!((normalized_damerau_levenshtein("levenshtein", "löwenbräu") - 0.27272).abs() < 0.00001);
    }

    #[test]
    fn normalized_damerau_levenshtein_for_empty_strings() {
        assert!((normalized_damerau_levenshtein("", "") - 1.0).abs() < 0.00001);
    }

    #[test]
    fn normalized_damerau_levenshtein_first_empty() {
        assert!(normalized_damerau_levenshtein("", "flower").abs() < 0.00001);
    }

    #[test]
    fn normalized_damerau_levenshtein_second_empty() {
        assert!(normalized_damerau_levenshtein("tree", "").abs() < 0.00001);
    }

    #[test]
    fn normalized_damerau_levenshtein_identical_strings() {
        assert!((normalized_damerau_levenshtein("sunglasses", "sunglasses") - 1.0).abs() < 0.00001);
    }
}
