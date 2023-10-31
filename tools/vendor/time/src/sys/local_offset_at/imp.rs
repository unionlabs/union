//! A fallback for any OS not covered.

use crate::{OffsetDateTime, UtcOffset};

#[allow(clippy::missing_docs_in_private_items)]
pub(super) fn local_offset_at(_datetime: OffsetDateTime) -> Option<UtcOffset> {
    None
}
