// Copyright 2015 The Servo Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! 3.3.3 Preparations for Implicit Processing
//!
//! <http://www.unicode.org/reports/tr9/#Preparations_for_Implicit_Processing>

use alloc::vec::Vec;
use core::cmp::max;
use core::ops::Range;

use super::level::Level;
use super::BidiClass::{self, *};

/// A maximal substring of characters with the same embedding level.
///
/// Represented as a range of byte indices.
pub type LevelRun = Range<usize>;

/// Output of `isolating_run_sequences` (steps X9-X10)
#[derive(Debug, PartialEq)]
pub struct IsolatingRunSequence {
    pub runs: Vec<LevelRun>,
    pub sos: BidiClass, // Start-of-sequence type.
    pub eos: BidiClass, // End-of-sequence type.
}

/// Compute the set of isolating run sequences.
///
/// An isolating run sequence is a maximal sequence of level runs such that for all level runs
/// except the last one in the sequence, the last character of the run is an isolate initiator
/// whose matching PDI is the first character of the next level run in the sequence.
///
/// Note: This function does *not* return the sequences in order by their first characters.
#[cfg_attr(feature = "flame_it", flamer::flame)]
pub fn isolating_run_sequences(
    para_level: Level,
    original_classes: &[BidiClass],
    levels: &[Level],
) -> Vec<IsolatingRunSequence> {
    let runs = level_runs(levels, original_classes);

    // Compute the set of isolating run sequences.
    // <http://www.unicode.org/reports/tr9/#BD13>
    let mut sequences = Vec::with_capacity(runs.len());

    // When we encounter an isolate initiator, we push the current sequence onto the
    // stack so we can resume it after the matching PDI.
    let mut stack = vec![Vec::new()];

    for run in runs {
        assert!(run.len() > 0);
        assert!(!stack.is_empty());

        let start_class = original_classes[run.start];
        let end_class = original_classes[run.end - 1];

        let mut sequence = if start_class == PDI && stack.len() > 1 {
            // Continue a previous sequence interrupted by an isolate.
            stack.pop().unwrap()
        } else {
            // Start a new sequence.
            Vec::new()
        };

        sequence.push(run);

        if let RLI | LRI | FSI = end_class {
            // Resume this sequence after the isolate.
            stack.push(sequence);
        } else {
            // This sequence is finished.
            sequences.push(sequence);
        }
    }
    // Pop any remaning sequences off the stack.
    sequences.extend(stack.into_iter().rev().filter(|seq| !seq.is_empty()));

    // Determine the `sos` and `eos` class for each sequence.
    // <http://www.unicode.org/reports/tr9/#X10>
    sequences
        .into_iter()
        .map(|sequence: Vec<LevelRun>| {
            assert!(!sequence.is_empty());

            let mut result = IsolatingRunSequence {
                runs: sequence,
                sos: L,
                eos: L,
            };

            let start_of_seq = result.runs[0].start;
            let runs_len = result.runs.len();
            let end_of_seq = result.runs[runs_len - 1].end;

            // > (not counting characters removed by X9)
            let seq_level = result
                .iter_forwards_from(start_of_seq, 0)
                .filter(|i| not_removed_by_x9(&original_classes[*i]))
                .map(|i| levels[i])
                .next()
                .unwrap_or(levels[start_of_seq]);

            // XXXManishearth the spec talks of a start and end level,
            // but for a given IRS the two should be equivalent, yes?
            let end_level = result
                .iter_backwards_from(end_of_seq, runs_len - 1)
                .filter(|i| not_removed_by_x9(&original_classes[*i]))
                .map(|i| levels[i])
                .next()
                .unwrap_or(levels[end_of_seq - 1]);

            #[cfg(test)]
            for run in result.runs.clone() {
                for idx in run {
                    if not_removed_by_x9(&original_classes[idx]) {
                        assert_eq!(seq_level, levels[idx]);
                    }
                }
            }

            // Get the level of the last non-removed char before the runs.
            let pred_level = match original_classes[..start_of_seq]
                .iter()
                .rposition(not_removed_by_x9)
            {
                Some(idx) => levels[idx],
                None => para_level,
            };

            // Get the last non-removed character to check if it is an isolate initiator.
            // The spec calls for an unmatched one, but matched isolate initiators
            // will never be at the end of a level run (otherwise there would be more to the run).
            // We unwrap_or(BN) because BN marks removed classes and it won't matter for the check.
            let last_non_removed = original_classes[..end_of_seq]
                .iter()
                .copied()
                .rev()
                .find(not_removed_by_x9)
                .unwrap_or(BN);

            // Get the level of the next non-removed char after the runs.
            let succ_level = if let RLI | LRI | FSI = last_non_removed {
                para_level
            } else {
                match original_classes[end_of_seq..]
                    .iter()
                    .position(not_removed_by_x9)
                {
                    Some(idx) => levels[end_of_seq + idx],
                    None => para_level,
                }
            };

            result.sos = max(seq_level, pred_level).bidi_class();
            result.eos = max(end_level, succ_level).bidi_class();
            result
        })
        .collect()
}

impl IsolatingRunSequence {
    /// Returns the full range of text represented by this isolating run sequence
    pub(crate) fn text_range(&self) -> Range<usize> {
        if let (Some(start), Some(end)) = (self.runs.first(), self.runs.last()) {
            start.start..end.end
        } else {
            return 0..0;
        }
    }

    /// Given a text-relative position `pos` and an index of the level run it is in,
    /// produce an iterator of all characters after and pos (`pos..`) that are in this
    /// run sequence
    pub(crate) fn iter_forwards_from(
        &self,
        pos: usize,
        level_run_index: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        let runs = &self.runs[level_run_index..];

        // Check that it is in range
        // (we can't use contains() since we want an inclusive range)
        #[cfg(feature = "std")]
        debug_assert!(runs[0].start <= pos && pos <= runs[0].end);

        (pos..runs[0].end).chain(runs[1..].iter().flat_map(Clone::clone))
    }

    /// Given a text-relative position `pos` and an index of the level run it is in,
    /// produce an iterator of all characters before and excludingpos (`..pos`) that are in this
    /// run sequence
    pub(crate) fn iter_backwards_from(
        &self,
        pos: usize,
        level_run_index: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        let prev_runs = &self.runs[..level_run_index];
        let current = &self.runs[level_run_index];

        // Check that it is in range
        // (we can't use contains() since we want an inclusive range)
        #[cfg(feature = "std")]
        debug_assert!(current.start <= pos && pos <= current.end);

        (current.start..pos)
            .rev()
            .chain(prev_runs.iter().rev().flat_map(Clone::clone))
    }
}

/// Finds the level runs in a paragraph.
///
/// <http://www.unicode.org/reports/tr9/#BD7>
fn level_runs(levels: &[Level], original_classes: &[BidiClass]) -> Vec<LevelRun> {
    assert_eq!(levels.len(), original_classes.len());

    let mut runs = Vec::new();
    if levels.is_empty() {
        return runs;
    }

    let mut current_run_level = levels[0];
    let mut current_run_start = 0;
    for i in 1..levels.len() {
        if !removed_by_x9(original_classes[i]) && levels[i] != current_run_level {
            // End the last run and start a new one.
            runs.push(current_run_start..i);
            current_run_level = levels[i];
            current_run_start = i;
        }
    }
    runs.push(current_run_start..levels.len());

    runs
}

/// Should this character be ignored in steps after X9?
///
/// <http://www.unicode.org/reports/tr9/#X9>
pub fn removed_by_x9(class: BidiClass) -> bool {
    match class {
        RLE | LRE | RLO | LRO | PDF | BN => true,
        _ => false,
    }
}

// For use as a predicate for `position` / `rposition`
pub fn not_removed_by_x9(class: &BidiClass) -> bool {
    !removed_by_x9(*class)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_runs() {
        assert_eq!(level_runs(&Level::vec(&[]), &[]), &[]);
        assert_eq!(
            level_runs(&Level::vec(&[0, 0, 0, 1, 1, 2, 0, 0]), &[L; 8]),
            &[0..3, 3..5, 5..6, 6..8]
        );
    }

    // From <http://www.unicode.org/reports/tr9/#BD13>
    #[rustfmt::skip]
    #[test]
    fn test_isolating_run_sequences() {

        // == Example 1 ==
        // text1·RLE·text2·PDF·RLE·text3·PDF·text4
        // index        0    1  2    3    4  5    6  7
        let classes = &[L, RLE, L, PDF, RLE, L, PDF, L];
        let levels =  &[0,   1, 1,   1,   1, 1,   1, 0];
        let para_level = Level::ltr();
        let mut sequences = isolating_run_sequences(para_level, classes, &Level::vec(levels));
        sequences.sort_by(|a, b| a.runs[0].clone().cmp(b.runs[0].clone()));
        assert_eq!(
            sequences.iter().map(|s| s.runs.clone()).collect::<Vec<_>>(),
            vec![vec![0..2], vec![2..7], vec![7..8]]
        );

        // == Example 2 ==
        // text1·RLI·text2·PDI·RLI·text3·PDI·text4
        // index        0    1  2    3    4  5    6  7
        let classes = &[L, RLI, L, PDI, RLI, L, PDI, L];
        let levels =  &[0,   0, 1,   0,   0, 1,   0, 0];
        let para_level = Level::ltr();
        let mut sequences = isolating_run_sequences(para_level, classes, &Level::vec(levels));
        sequences.sort_by(|a, b| a.runs[0].clone().cmp(b.runs[0].clone()));
        assert_eq!(
            sequences.iter().map(|s| s.runs.clone()).collect::<Vec<_>>(),
            vec![vec![0..2, 3..5, 6..8], vec![2..3], vec![5..6]]
        );

        // == Example 3 ==
        // text1·RLI·text2·LRI·text3·RLE·text4·PDF·text5·PDI·text6·PDI·text7
        // index        0    1  2    3  4    5  6    7  8    9  10  11  12
        let classes = &[L, RLI, L, LRI, L, RLE, L, PDF, L, PDI, L, PDI,  L];
        let levels =  &[0,   0, 1,   1, 2,   3, 3,   3, 2,   1, 1,   0,  0];
        let para_level = Level::ltr();
        let mut sequences = isolating_run_sequences(para_level, classes, &Level::vec(levels));
        sequences.sort_by(|a, b| a.runs[0].clone().cmp(b.runs[0].clone()));
        assert_eq!(
            sequences.iter().map(|s| s.runs.clone()).collect::<Vec<_>>(),
            vec![vec![0..2, 11..13], vec![2..4, 9..11], vec![4..6], vec![6..8], vec![8..9]]
        );
    }

    // From <http://www.unicode.org/reports/tr9/#X10>
    #[rustfmt::skip]
    #[test]
    fn test_isolating_run_sequences_sos_and_eos() {

        // == Example 1 ==
        // text1·RLE·text2·LRE·text3·PDF·text4·PDF·RLE·text5·PDF·text6
        // index        0    1  2    3  4    5  6    7    8  9   10  11
        let classes = &[L, RLE, L, LRE, L, PDF, L, PDF, RLE, L, PDF,  L];
        let levels =  &[0,   1, 1,   2, 2,   2, 1,   1,   1, 1,   1,  0];
        let para_level = Level::ltr();
        let mut sequences = isolating_run_sequences(para_level, classes, &Level::vec(levels));
        sequences.sort_by(|a, b| a.runs[0].clone().cmp(b.runs[0].clone()));

        // text1
        assert_eq!(
            &sequences[0],
            &IsolatingRunSequence {
                runs: vec![0..2],
                sos: L,
                eos: R,
            }
        );

        // text2
        assert_eq!(
            &sequences[1],
            &IsolatingRunSequence {
                runs: vec![2..4],
                sos: R,
                eos: L,
            }
        );

        // text3
        assert_eq!(
            &sequences[2],
            &IsolatingRunSequence {
                runs: vec![4..6],
                sos: L,
                eos: L,
            }
        );

        // text4 text5
        assert_eq!(
            &sequences[3],
            &IsolatingRunSequence {
                runs: vec![6..11],
                sos: L,
                eos: R,
            }
        );

        // text6
        assert_eq!(
            &sequences[4],
            &IsolatingRunSequence {
                runs: vec![11..12],
                sos: R,
                eos: L,
            }
        );

        // == Example 2 ==
        // text1·RLI·text2·LRI·text3·PDI·text4·PDI·RLI·text5·PDI·text6
        // index        0    1  2    3  4    5  6    7    8  9   10  11
        let classes = &[L, RLI, L, LRI, L, PDI, L, PDI, RLI, L, PDI,  L];
        let levels =  &[0,   0, 1,   1, 2,   1, 1,   0,   0, 1,   0,  0];
        let para_level = Level::ltr();
        let mut sequences = isolating_run_sequences(para_level, classes, &Level::vec(levels));
        sequences.sort_by(|a, b| a.runs[0].clone().cmp(b.runs[0].clone()));

        // text1·RLI·PDI·RLI·PDI·text6
        assert_eq!(
            &sequences[0],
            &IsolatingRunSequence {
                runs: vec![0..2, 7..9, 10..12],
                sos: L,
                eos: L,
            }
        );

        // text2·LRI·PDI·text4
        assert_eq!(
            &sequences[1],
            &IsolatingRunSequence {
                runs: vec![2..4, 5..7],
                sos: R,
                eos: R,
            }
        );

        // text3
        assert_eq!(
            &sequences[2],
            &IsolatingRunSequence {
                runs: vec![4..5],
                sos: L,
                eos: L,
            }
        );

        // text5
        assert_eq!(
            &sequences[3],
            &IsolatingRunSequence {
                runs: vec![9..10],
                sos: R,
                eos: R,
            }
        );
    }

    #[test]
    fn test_removed_by_x9() {
        let rem_classes = &[RLE, LRE, RLO, LRO, PDF, BN];
        let not_classes = &[L, RLI, AL, LRI, PDI];
        for x in rem_classes {
            assert_eq!(removed_by_x9(*x), true);
        }
        for x in not_classes {
            assert_eq!(removed_by_x9(*x), false);
        }
    }

    #[test]
    fn test_not_removed_by_x9() {
        let non_x9_classes = &[L, R, AL, EN, ES, ET, AN, CS, NSM, B, S, WS, ON, LRI, RLI, FSI, PDI];
        for x in non_x9_classes {
            assert_eq!(not_removed_by_x9(&x), true);
        }
    }
}
