use std::{
    fmt::Display,
    fs::File,
    ops::{Bound, RangeBounds},
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::Buf;
use memmap2::Mmap;

#[derive(Debug, Clone)]
pub(crate) struct MmappedSlice {
    mmap: Arc<Mmap>,
    start: usize,
    end: usize,
}

impl MmappedSlice {
    pub fn from_path(path: &Path) -> Result<Self, MmapError> {
        let f = File::open(path).map_err(|error| MmapError::FileOpen {
            error,
            path: path.to_path_buf(),
        })?;
        MmappedSlice::from_file(&f)
    }

    pub fn from_file(file: &File) -> Result<Self, MmapError> {
        unsafe {
            let mmap = Mmap::map(file).map_err(MmapError::Map)?;
            let end = mmap.len();
            Ok(MmappedSlice {
                mmap: Arc::new(mmap),
                start: 0,
                end,
            })
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[self.start..self.end]
    }

    #[inline]
    #[track_caller]
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        let (start, end) = bounds(self.start, self.end, range);
        MmappedSlice {
            mmap: Arc::clone(&self.mmap),
            start,
            end,
        }
    }
}

#[track_caller]
fn bounds(
    original_start: usize,
    original_end: usize,
    range: impl RangeBounds<usize>,
) -> (usize, usize) {
    let start_offset = match range.start_bound() {
        Bound::Included(&index) => index,
        Bound::Excluded(index) => index.saturating_sub(1),
        Bound::Unbounded => 0,
    };
    let start = original_start + start_offset;

    let end = match range.end_bound() {
        Bound::Included(index) => original_start + index.saturating_add(1),
        Bound::Excluded(&index) => original_start + index,
        Bound::Unbounded => original_end,
    };

    assert!(start <= end, "{start} <= {end}");
    assert!(
        start >= original_start,
        "Start offset out of bounds: {start} >= {original_start}"
    );
    assert!(
        end <= original_end,
        "End offset out of bounds: {end} <= {original_end}"
    );

    (start, end)
}

impl Buf for MmappedSlice {
    fn remaining(&self) -> usize {
        self.as_slice().len()
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, cnt: usize) {
        debug_assert!(cnt <= self.remaining());
        self.start += cnt;
    }
}

/// Errors that may occur when using one of the mmap-based implementations of
/// [`TryFrom`].
#[derive(Debug)]
pub enum MmapError {
    /// Unable to open the file.
    FileOpen {
        error: std::io::Error,
        path: PathBuf,
    },
    /// Mapping the file into memory failed.
    Map(std::io::Error),
}

impl Display for MmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MmapError::FileOpen { path, .. } => write!(f, "Unable to open \"{}\"", path.display()),
            MmapError::Map(_) => write!(f, "Unable to map the file into memory"),
        }
    }
}

impl std::error::Error for MmapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MmapError::FileOpen { error, .. } | MmapError::Map(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn full_range() {
        let (start, end) = bounds(1, 10, ..);

        assert_eq!(start, 1);
        assert_eq!(end, 10);
    }

    #[test]
    fn range_to() {
        let (start, end) = bounds(1, 10, ..5);

        assert_eq!(start, 1);
        assert_eq!(end, 1 + 5);
    }

    #[test]
    fn range_to_inclusive() {
        let (start, end) = bounds(1, 10, ..=5);

        assert_eq!(start, 1);
        assert_eq!(end, 1 + 6);
    }

    #[test]
    fn range_from() {
        let (start, end) = bounds(1, 10, 5..);

        assert_eq!(start, 1 + 5);
        assert_eq!(end, 10);
    }

    #[test]
    fn range() {
        let (start, end) = bounds(1, 10, 5..8);

        assert_eq!(start, 1 + 5);
        assert_eq!(end, 1 + 8);
    }

    #[test]
    fn range_at_end() {
        let (start, end) = bounds(1, 10, 5..9);

        assert_eq!(start, 1 + 5);
        assert_eq!(end, 1 + 9);
    }

    #[test]
    fn range_at_start() {
        let (start, end) = bounds(1, 10, 1..5);

        assert_eq!(start, 1 + 1);
        assert_eq!(end, 1 + 5);
    }

    #[test]
    fn range_inclusive() {
        let (start, end) = bounds(1, 10, 1..=5);

        assert_eq!(start, 1 + 1);
        assert_eq!(end, 1 + 5 + 1);
    }

    #[test]
    fn range_inclusive_at_end() {
        let (start, end) = bounds(1, 10, 5..=8);

        assert_eq!(start, 1 + 5);
        assert_eq!(end, 1 + 8 + 1);
    }

    #[test]
    fn simple_mmap() {
        let mut temp = tempfile::tempfile().unwrap();
        let content = b"Hello, World!";
        temp.write_all(content).unwrap();

        let mmap = MmappedSlice::from_file(&temp).unwrap();

        assert_eq!(mmap.as_slice(), content);
    }

    #[test]
    fn slice_mmap() {
        let mut temp = tempfile::tempfile().unwrap();
        let content = b"Hello, World!";
        temp.write_all(content).unwrap();
        let mmap = MmappedSlice::from_file(&temp).unwrap();

        let slice = mmap.slice(..5);

        assert_eq!(slice.as_slice(), b"Hello");
    }

    #[test]
    fn slicing_is_relative_to_the_slice_not_the_overall_file() {
        let mut temp = tempfile::tempfile().unwrap();
        let content = "Hello, World!";
        temp.write_all(content.as_ref()).unwrap();
        let mmap = MmappedSlice::from_file(&temp).unwrap();
        let slice = mmap.slice(3..);

        let sub_slice = slice.slice(4..7);

        assert_eq!(
            std::str::from_utf8(sub_slice.as_slice()).unwrap(),
            &content[3 + 4..3 + 7]
        );
    }
}
