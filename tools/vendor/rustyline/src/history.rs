//! History API

use fd_lock::RwLock;
use log::{debug, warn};
use std::collections::vec_deque;
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::SeekFrom;
use std::iter::DoubleEndedIterator;
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Result;
use crate::config::{Config, HistoryDuplicates};

/// Search direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchDirection {
    /// Search history forward
    Forward,
    /// Search history backward
    Reverse,
}

/// History search result
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SearchResult<'a> {
    /// history entry
    pub entry: &'a str,
    /// history index
    pub idx: usize,
    /// match position in `entry`
    pub pos: usize,
}

/// HistoryEntry: text + timestamp
/// TODO Make possible to customize how history is stored / loaded.
/// https://github.com/kkawakam/rustyline/issues/442
/// https://github.com/kkawakam/rustyline/issues/127
/// See https://python-prompt-toolkit.readthedocs.io/en/master/pages/reference.html#prompt_toolkit.history.History abstract methods

/// Current state of the history.
#[derive(Default)]
pub struct History {
    entries: VecDeque<String>,
    max_len: usize,
    pub(crate) ignore_space: bool,
    pub(crate) ignore_dups: bool,
    /// Number of entries inputed by user and not saved yet
    new_entries: usize,
    /// last path used by either `load` or `save`
    path_info: Option<PathInfo>,
}

/// Last histo path, modified timestamp and size
struct PathInfo(PathBuf, SystemTime, usize);

impl History {
    // New multiline-aware history files start with `#V2\n` and have newlines
    // and backslashes escaped in them.
    const FILE_VERSION_V2: &'static str = "#V2";

    /// Default constructor
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    /// Customized constructor with:
    /// - `Config::max_history_size()`,
    /// - `Config::history_ignore_space()`,
    /// - `Config::history_duplicates()`.
    pub fn with_config(config: Config) -> Self {
        Self {
            entries: VecDeque::new(),
            max_len: config.max_history_size(),
            ignore_space: config.history_ignore_space(),
            ignore_dups: config.history_duplicates() == HistoryDuplicates::IgnoreConsecutive,
            new_entries: 0,
            path_info: None,
        }
    }

    /// Return the history entry at position `index`, starting from 0.
    pub fn get(&self, index: usize) -> Option<&String> {
        self.entries.get(index)
    }

    /// Return the last history entry (i.e. previous command)
    pub fn last(&self) -> Option<&String> {
        self.entries.back()
    }

    /// Add a new entry in the history.
    pub fn add<S: AsRef<str> + Into<String>>(&mut self, line: S) -> bool {
        if self.max_len == 0 {
            return false;
        }
        if line.as_ref().is_empty()
            || (self.ignore_space
                && line
                    .as_ref()
                    .chars()
                    .next()
                    .map_or(true, char::is_whitespace))
        {
            return false;
        }
        if self.ignore_dups {
            if let Some(s) = self.entries.back() {
                if s == line.as_ref() {
                    return false;
                }
            }
        }
        if self.entries.len() == self.max_len {
            self.entries.pop_front();
        }
        self.entries.push_back(line.into());
        self.new_entries = self.new_entries.saturating_add(1).min(self.len());
        true
    }

    /// Return the number of entries in the history.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return true if the history has no entry.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Set the maximum length for the history. This function can be called even
    /// if there is already some history, the function will make sure to retain
    /// just the latest `len` elements if the new history length value is
    /// smaller than the amount of items already inside the history.
    ///
    /// Like [stifle_history](http://tiswww.case.edu/php/chet/readline/history.html#IDX11).
    pub fn set_max_len(&mut self, len: usize) {
        self.max_len = len;
        if self.len() > len {
            self.entries.drain(..self.len() - len);
            self.new_entries = self.new_entries.min(len);
        }
    }

    /// Save the history in the specified file.
    // TODO history_truncate_file
    // https://tiswww.case.edu/php/chet/readline/history.html#IDX31
    pub fn save<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        if self.is_empty() || self.new_entries == 0 {
            return Ok(());
        }
        let path = path.as_ref();
        let old_umask = umask();
        let f = File::create(path);
        restore_umask(old_umask);
        let file = f?;
        let mut lock = RwLock::new(file);
        let lock_guard = lock.write()?;
        self.save_to(&lock_guard, false)?;
        self.new_entries = 0;
        self.update_path(path, &lock_guard, self.len())
    }

    fn save_to(&mut self, file: &File, append: bool) -> Result<()> {
        use std::io::{BufWriter, Write};

        fix_perm(file);
        let mut wtr = BufWriter::new(file);
        let first_new_entry = if append {
            self.entries.len().saturating_sub(self.new_entries)
        } else {
            wtr.write_all(Self::FILE_VERSION_V2.as_bytes())?;
            wtr.write_all(b"\n")?;
            0
        };
        for entry in self.entries.iter().skip(first_new_entry) {
            let mut bytes = entry.as_bytes();
            while let Some(i) = memchr::memchr2(b'\\', b'\n', bytes) {
                wtr.write_all(&bytes[..i])?;
                if bytes[i] == b'\n' {
                    wtr.write_all(b"\\n")?; // escaped line feed
                } else {
                    debug_assert_eq!(bytes[i], b'\\');
                    wtr.write_all(b"\\\\")?; // escaped backslash
                }
                bytes = &bytes[i + 1..];
            }
            wtr.write_all(bytes)?; // remaining bytes with no \n or \
            wtr.write_all(b"\n")?;
        }
        // https://github.com/rust-lang/rust/issues/32677#issuecomment-204833485
        wtr.flush()?;
        Ok(())
    }

    /// Append new entries in the specified file.
    // Like [append_history](http://tiswww.case.edu/php/chet/readline/history.html#IDX30).
    pub fn append<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        use std::io::Seek;

        if self.is_empty() || self.new_entries == 0 {
            return Ok(());
        }
        let path = path.as_ref();
        if !path.exists() || self.new_entries == self.max_len {
            return self.save(path);
        }
        let file = OpenOptions::new().write(true).read(true).open(path)?;
        let mut lock = RwLock::new(file);
        let mut lock_guard = lock.write()?;
        if self.can_just_append(path, &lock_guard)? {
            lock_guard.seek(SeekFrom::End(0))?;
            self.save_to(&lock_guard, true)?;
            let size = self
                .path_info
                .as_ref()
                .unwrap()
                .2
                .saturating_add(self.new_entries);
            self.new_entries = 0;
            return self.update_path(path, &lock_guard, size);
        }
        // we may need to truncate file before appending new entries
        let mut other = Self {
            entries: VecDeque::new(),
            max_len: self.max_len,
            ignore_space: self.ignore_space,
            ignore_dups: self.ignore_dups,
            new_entries: 0,
            path_info: None,
        };
        other.load_from(&lock_guard)?;
        let first_new_entry = self.entries.len().saturating_sub(self.new_entries);
        for entry in self.entries.iter().skip(first_new_entry) {
            other.add(entry);
        }
        lock_guard.seek(SeekFrom::Start(0))?;
        lock_guard.set_len(0)?; // if new size < old size
        other.save_to(&lock_guard, false)?;
        self.update_path(path, &lock_guard, other.len())?;
        self.new_entries = 0;
        Ok(())
    }

    /// Load the history from the specified file.
    ///
    /// # Errors
    /// Will return `Err` if path does not already exist or could not be read.
    pub fn load<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let lock = RwLock::new(file);
        let lock_guard = lock.read()?;
        let len = self.len();
        if self.load_from(&lock_guard)? {
            self.update_path(path, &lock_guard, self.len() - len)
        } else {
            // discard old version on next save
            self.path_info = None;
            Ok(())
        }
    }

    fn load_from(&mut self, file: &File) -> Result<bool> {
        use std::io::{BufRead, BufReader};

        let rdr = BufReader::new(file);
        let mut lines = rdr.lines();
        let mut v2 = false;
        if let Some(first) = lines.next() {
            let line = first?;
            if line == Self::FILE_VERSION_V2 {
                v2 = true;
            } else {
                self.add(line);
            }
        }
        let mut appendable = v2;
        for line in lines {
            let mut line = line?;
            if line.is_empty() {
                continue;
            }
            if v2 {
                let mut copy = None; // lazily copy line if unescaping is needed
                let mut str = line.as_str();
                while let Some(i) = str.find('\\') {
                    if copy.is_none() {
                        copy = Some(String::with_capacity(line.len()));
                    }
                    let s = copy.as_mut().unwrap();
                    s.push_str(&str[..i]);
                    let j = i + 1; // escaped char idx
                    let b = if j < str.len() {
                        str.as_bytes()[j]
                    } else {
                        0 // unexpected if History::save works properly
                    };
                    match b {
                        b'n' => {
                            s.push('\n'); // unescaped line feed
                        }
                        b'\\' => {
                            s.push('\\'); // unescaped back slash
                        }
                        _ => {
                            // only line feed and back slash should have been escaped
                            warn!(target: "rustyline", "bad escaped line: {}", line);
                            copy = None;
                            break;
                        }
                    }
                    str = &str[j + 1..];
                }
                if let Some(mut s) = copy {
                    s.push_str(str); // remaining bytes with no escaped char
                    line = s;
                }
            }
            appendable &= self.add(line); // TODO truncate to MAX_LINE
        }
        self.new_entries = 0; // TODO we may lost new entries if loaded lines < max_len
        Ok(appendable)
    }

    fn update_path(&mut self, path: &Path, file: &File, size: usize) -> Result<()> {
        let modified = file.metadata()?.modified()?;
        if let Some(PathInfo(
            ref mut previous_path,
            ref mut previous_modified,
            ref mut previous_size,
        )) = self.path_info
        {
            if previous_path.as_path() != path {
                *previous_path = path.to_owned();
            }
            *previous_modified = modified;
            *previous_size = size;
        } else {
            self.path_info = Some(PathInfo(path.to_owned(), modified, size));
        }
        debug!(target: "rustyline", "PathInfo({:?}, {:?}, {})", path, modified, size);
        Ok(())
    }

    fn can_just_append(&self, path: &Path, file: &File) -> Result<bool> {
        if let Some(PathInfo(ref previous_path, ref previous_modified, ref previous_size)) =
            self.path_info
        {
            if previous_path.as_path() != path {
                debug!(target: "rustyline", "cannot append: {:?} <> {:?}", previous_path, path);
                return Ok(false);
            }
            let modified = file.metadata()?.modified()?;
            if *previous_modified != modified
                || self.max_len <= *previous_size
                || self.max_len < (*previous_size).saturating_add(self.new_entries)
            {
                debug!(target: "rustyline", "cannot append: {:?} < {:?} or {} < {} + {}",
                       previous_modified, modified, self.max_len, previous_size, self.new_entries);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.new_entries = 0;
    }

    /// Search history (start position inclusive [0, len-1]).
    ///
    /// Return the absolute index of the nearest history entry that matches
    /// `term`.
    ///
    /// Return None if no entry contains `term` between [start, len -1] for
    /// forward search
    /// or between [0, start] for reverse search.
    pub fn search(&self, term: &str, start: usize, dir: SearchDirection) -> Option<SearchResult> {
        #[cfg(not(feature = "case_insensitive_history_search"))]
        {
            let test = |entry: &str| entry.find(term);
            self.search_match(term, start, dir, test)
        }
        #[cfg(feature = "case_insensitive_history_search")]
        {
            use regex::{escape, RegexBuilder};
            if let Ok(re) = RegexBuilder::new(&escape(term))
                .case_insensitive(true)
                .build()
            {
                let test = |entry: &str| re.find(entry).map(|m| m.start());
                self.search_match(term, start, dir, test)
            } else {
                None
            }
        }
    }

    /// Anchored search
    pub fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> Option<SearchResult> {
        #[cfg(not(feature = "case_insensitive_history_search"))]
        {
            let test = |entry: &str| {
                if entry.starts_with(term) {
                    Some(term.len())
                } else {
                    None
                }
            };
            self.search_match(term, start, dir, test)
        }
        #[cfg(feature = "case_insensitive_history_search")]
        {
            use regex::{escape, RegexBuilder};
            if let Ok(re) = RegexBuilder::new(&escape(term))
                .case_insensitive(true)
                .build()
            {
                let test = |entry: &str| {
                    re.find(entry)
                        .and_then(|m| if m.start() == 0 { Some(m) } else { None })
                        .map(|m| m.end())
                };
                self.search_match(term, start, dir, test)
            } else {
                None
            }
        }
    }

    fn search_match<F>(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
        test: F,
    ) -> Option<SearchResult>
    where
        F: Fn(&str) -> Option<usize>,
    {
        if term.is_empty() || start >= self.len() {
            return None;
        }
        match dir {
            SearchDirection::Reverse => {
                for (idx, entry) in self
                    .entries
                    .iter()
                    .rev()
                    .skip(self.entries.len() - 1 - start)
                    .enumerate()
                {
                    if let Some(cursor) = test(entry) {
                        return Some(SearchResult {
                            idx: start - idx,
                            entry,
                            pos: cursor,
                        });
                    }
                }
                None
            }
            SearchDirection::Forward => {
                for (idx, entry) in self.entries.iter().skip(start).enumerate() {
                    if let Some(cursor) = test(entry) {
                        return Some(SearchResult {
                            idx: idx + start,
                            entry,
                            pos: cursor,
                        });
                    }
                }
                None
            }
        }
    }

    /// Return a forward iterator.
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.entries.iter())
    }
}

impl Index<usize> for History {
    type Output = String;

    fn index(&self, index: usize) -> &String {
        &self.entries[index]
    }
}

impl<'a> IntoIterator for &'a History {
    type IntoIter = Iter<'a>;
    type Item = &'a String;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

/// History iterator.
pub struct Iter<'a>(vec_deque::Iter<'a, String>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<&'a String> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<&'a String> {
        self.0.next_back()
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(windows, target_arch = "wasm32"))] {
        fn umask() -> u16 {
            0
        }

        fn restore_umask(_: u16) {}

        fn fix_perm(_: &File) {}
    } else if #[cfg(unix)] {
        fn umask() -> libc::mode_t {
            unsafe { libc::umask(libc::S_IXUSR | libc::S_IRWXG | libc::S_IRWXO) }
        }

        fn restore_umask(old_umask: libc::mode_t) {
            unsafe {
                libc::umask(old_umask);
            }
        }

        fn fix_perm(file: &File) {
            use std::os::unix::io::AsRawFd;
            unsafe {
                libc::fchmod(file.as_raw_fd(), libc::S_IRUSR | libc::S_IWUSR);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{History, SearchDirection, SearchResult};
    use crate::config::Config;
    use crate::Result;

    fn init() -> History {
        let mut history = History::new();
        assert!(history.add("line1"));
        assert!(history.add("line2"));
        assert!(history.add("line3"));
        history
    }

    #[test]
    fn new() {
        let history = History::new();
        assert_eq!(0, history.entries.len());
    }

    #[test]
    fn add() {
        let config = Config::builder().history_ignore_space(true).build();
        let mut history = History::with_config(config);
        assert_eq!(config.max_history_size(), history.max_len);
        assert!(history.add("line1"));
        assert!(history.add("line2"));
        assert!(!history.add("line2"));
        assert!(!history.add(""));
        assert!(!history.add(" line3"));
    }

    #[test]
    fn set_max_len() {
        let mut history = init();
        history.set_max_len(1);
        assert_eq!(1, history.entries.len());
        assert_eq!(Some(&"line3".to_owned()), history.last());
    }

    #[test]
    #[cfg_attr(miri, ignore)] // unsupported operation: `getcwd` not available when isolation is enabled
    fn save() -> Result<()> {
        check_save("line\nfour \\ abc")
    }

    #[test]
    #[cfg_attr(miri, ignore)] // unsupported operation: `open` not available when isolation is enabled
    fn save_windows_path() -> Result<()> {
        let path = "cd source\\repos\\forks\\nushell\\";
        check_save(path)
    }

    fn check_save(line: &str) -> Result<()> {
        let mut history = init();
        assert!(history.add(line));
        let tf = tempfile::NamedTempFile::new()?;

        history.save(tf.path())?;
        let mut history2 = History::new();
        history2.load(tf.path())?;
        for (a, b) in history.entries.iter().zip(history2.entries.iter()) {
            assert_eq!(a, b);
        }
        tf.close()?;
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // unsupported operation: `getcwd` not available when isolation is enabled
    fn load_legacy() -> Result<()> {
        use std::io::Write;
        let tf = tempfile::NamedTempFile::new()?;
        {
            let mut legacy = std::fs::File::create(tf.path())?;
            // Some data we'd accidentally corrupt if we got the version wrong
            let data = b"\
                test\\n \\abc \\123\n\
                123\\n\\\\n\n\
                abcde
            ";
            legacy.write_all(data)?;
            legacy.flush()?;
        }
        let mut history = History::new();
        history.load(tf.path())?;
        assert_eq!(history.entries[0], "test\\n \\abc \\123");
        assert_eq!(history.entries[1], "123\\n\\\\n");
        assert_eq!(history.entries[2], "abcde");

        tf.close()?;
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // unsupported operation: `getcwd` not available when isolation is enabled
    fn append() -> Result<()> {
        let mut history = init();
        let tf = tempfile::NamedTempFile::new()?;

        history.append(tf.path())?;

        let mut history2 = History::new();
        history2.load(tf.path())?;
        history2.add("line4");
        history2.append(tf.path())?;

        history.add("line5");
        history.append(tf.path())?;

        let mut history3 = History::new();
        history3.load(tf.path())?;
        assert_eq!(history3.len(), 5);

        tf.close()?;
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // unsupported operation: `getcwd` not available when isolation is enabled
    fn truncate() -> Result<()> {
        let tf = tempfile::NamedTempFile::new()?;

        let config = Config::builder().history_ignore_dups(false).build();
        let mut history = History::with_config(config);
        history.add("line1");
        history.add("line1");
        history.append(tf.path())?;

        let mut history = History::new();
        history.load(tf.path())?;
        history.add("l");
        history.append(tf.path())?;

        let mut history = History::new();
        history.load(tf.path())?;
        assert_eq!(history.len(), 2);
        assert_eq!(history.entries[1], "l");

        tf.close()?;
        Ok(())
    }

    #[test]
    fn search() {
        let history = init();
        assert_eq!(None, history.search("", 0, SearchDirection::Forward));
        assert_eq!(None, history.search("none", 0, SearchDirection::Forward));
        assert_eq!(None, history.search("line", 3, SearchDirection::Forward));

        assert_eq!(
            Some(SearchResult {
                idx: 0,
                entry: history.get(0).unwrap(),
                pos: 0
            }),
            history.search("line", 0, SearchDirection::Forward)
        );
        assert_eq!(
            Some(SearchResult {
                idx: 1,
                entry: history.get(1).unwrap(),
                pos: 0
            }),
            history.search("line", 1, SearchDirection::Forward)
        );
        assert_eq!(
            Some(SearchResult {
                idx: 2,
                entry: history.get(2).unwrap(),
                pos: 0
            }),
            history.search("line3", 1, SearchDirection::Forward)
        );
    }

    #[test]
    fn reverse_search() {
        let history = init();
        assert_eq!(None, history.search("", 2, SearchDirection::Reverse));
        assert_eq!(None, history.search("none", 2, SearchDirection::Reverse));
        assert_eq!(None, history.search("line", 3, SearchDirection::Reverse));

        assert_eq!(
            Some(SearchResult {
                idx: 2,
                entry: history.get(2).unwrap(),
                pos: 0
            }),
            history.search("line", 2, SearchDirection::Reverse)
        );
        assert_eq!(
            Some(SearchResult {
                idx: 1,
                entry: history.get(1).unwrap(),
                pos: 0
            }),
            history.search("line", 1, SearchDirection::Reverse)
        );
        assert_eq!(
            Some(SearchResult {
                idx: 0,
                entry: history.get(0).unwrap(),
                pos: 0
            }),
            history.search("line1", 1, SearchDirection::Reverse)
        );
    }

    #[test]
    #[cfg(feature = "case_insensitive_history_search")]
    fn anchored_search() {
        let history = init();
        assert_eq!(
            Some(SearchResult {
                idx: 2,
                entry: history.get(2).unwrap(),
                pos: 4
            }),
            history.starts_with("LiNe", 2, SearchDirection::Reverse)
        );
        assert_eq!(
            None,
            history.starts_with("iNe", 2, SearchDirection::Reverse)
        );
    }
}
