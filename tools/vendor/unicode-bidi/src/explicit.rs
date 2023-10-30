// Copyright 2015 The Servo Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! 3.3.2 Explicit Levels and Directions
//!
//! <http://www.unicode.org/reports/tr9/#Explicit_Levels_and_Directions>

use alloc::vec::Vec;

use super::char_data::{
    is_rtl,
    BidiClass::{self, *},
};
use super::level::Level;

/// Compute explicit embedding levels for one paragraph of text (X1-X8).
///
/// `processing_classes[i]` must contain the `BidiClass` of the char at byte index `i`,
/// for each char in `text`.
#[cfg_attr(feature = "flame_it", flamer::flame)]
pub fn compute(
    text: &str,
    para_level: Level,
    original_classes: &[BidiClass],
    levels: &mut [Level],
    processing_classes: &mut [BidiClass],
) {
    assert_eq!(text.len(), original_classes.len());

    // <http://www.unicode.org/reports/tr9/#X1>
    let mut stack = DirectionalStatusStack::new();
    stack.push(para_level, OverrideStatus::Neutral);

    let mut overflow_isolate_count = 0u32;
    let mut overflow_embedding_count = 0u32;
    let mut valid_isolate_count = 0u32;

    for (i, c) in text.char_indices() {
        match original_classes[i] {
            // Rules X2-X5c
            RLE | LRE | RLO | LRO | RLI | LRI | FSI => {
                let last_level = stack.last().level;

                // <https://www.unicode.org/reports/tr9/#Retaining_Explicit_Formatting_Characters>
                levels[i] = last_level;

                // X5a-X5c: Isolate initiators get the level of the last entry on the stack.
                let is_isolate = match original_classes[i] {
                    RLI | LRI | FSI => true,
                    _ => false,
                };
                if is_isolate {
                    // Redundant due to "Retaining explicit formatting characters" step.
                    // levels[i] = last_level;
                    match stack.last().status {
                        OverrideStatus::RTL => processing_classes[i] = R,
                        OverrideStatus::LTR => processing_classes[i] = L,
                        _ => {}
                    }
                }

                let new_level = if is_rtl(original_classes[i]) {
                    last_level.new_explicit_next_rtl()
                } else {
                    last_level.new_explicit_next_ltr()
                };
                if new_level.is_ok() && overflow_isolate_count == 0 && overflow_embedding_count == 0
                {
                    let new_level = new_level.unwrap();
                    stack.push(
                        new_level,
                        match original_classes[i] {
                            RLO => OverrideStatus::RTL,
                            LRO => OverrideStatus::LTR,
                            RLI | LRI | FSI => OverrideStatus::Isolate,
                            _ => OverrideStatus::Neutral,
                        },
                    );
                    if is_isolate {
                        valid_isolate_count += 1;
                    } else {
                        // The spec doesn't explicitly mention this step, but it is necessary.
                        // See the reference implementations for comparison.
                        levels[i] = new_level;
                    }
                } else if is_isolate {
                    overflow_isolate_count += 1;
                } else if overflow_isolate_count == 0 {
                    overflow_embedding_count += 1;
                }

                if !is_isolate {
                    // X9 +
                    // <https://www.unicode.org/reports/tr9/#Retaining_Explicit_Formatting_Characters>
                    // (PDF handled below)
                    processing_classes[i] = BN;
                }
            }

            // <http://www.unicode.org/reports/tr9/#X6a>
            PDI => {
                if overflow_isolate_count > 0 {
                    overflow_isolate_count -= 1;
                } else if valid_isolate_count > 0 {
                    overflow_embedding_count = 0;
                    loop {
                        // Pop everything up to and including the last Isolate status.
                        match stack.vec.pop() {
                            None
                            | Some(Status {
                                status: OverrideStatus::Isolate,
                                ..
                            }) => break,
                            _ => continue,
                        }
                    }
                    valid_isolate_count -= 1;
                }
                let last = stack.last();
                levels[i] = last.level;
                match last.status {
                    OverrideStatus::RTL => processing_classes[i] = R,
                    OverrideStatus::LTR => processing_classes[i] = L,
                    _ => {}
                }
            }

            // <http://www.unicode.org/reports/tr9/#X7>
            PDF => {
                if overflow_isolate_count > 0 {
                    // do nothing
                } else if overflow_embedding_count > 0 {
                    overflow_embedding_count -= 1;
                } else if stack.last().status != OverrideStatus::Isolate && stack.vec.len() >= 2 {
                    stack.vec.pop();
                }
                // <https://www.unicode.org/reports/tr9/#Retaining_Explicit_Formatting_Characters>
                levels[i] = stack.last().level;
                // X9 part of retaining explicit formatting characters.
                processing_classes[i] = BN;
            }

            // Nothing.
            // BN case moved down to X6, see <https://www.unicode.org/reports/tr9/#Retaining_Explicit_Formatting_Characters>
            B => {}

            // <http://www.unicode.org/reports/tr9/#X6>
            _ => {
                let last = stack.last();
                levels[i] = last.level;
                // This condition is not in the spec, but I am pretty sure that is a spec bug.
                // https://www.unicode.org/L2/L2023/23014-amd-to-uax9.pdf
                if original_classes[i] != BN {
                    match last.status {
                        OverrideStatus::RTL => processing_classes[i] = R,
                        OverrideStatus::LTR => processing_classes[i] = L,
                        _ => {}
                    }
                }
            }
        }

        // Handle multi-byte characters.
        for j in 1..c.len_utf8() {
            levels[i + j] = levels[i];
            processing_classes[i + j] = processing_classes[i];
        }
    }
}

/// Entries in the directional status stack:
struct Status {
    level: Level,
    status: OverrideStatus,
}

#[derive(PartialEq)]
enum OverrideStatus {
    Neutral,
    RTL,
    LTR,
    Isolate,
}

struct DirectionalStatusStack {
    vec: Vec<Status>,
}

impl DirectionalStatusStack {
    fn new() -> Self {
        DirectionalStatusStack {
            vec: Vec::with_capacity(Level::max_explicit_depth() as usize + 2),
        }
    }

    fn push(&mut self, level: Level, status: OverrideStatus) {
        self.vec.push(Status { level, status });
    }

    fn last(&self) -> &Status {
        self.vec.last().unwrap()
    }
}
