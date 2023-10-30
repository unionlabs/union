use crate::{Error, Protection, Region, Result};
use std::fs;

pub struct QueryIter {
  proc_maps: String,
  upper_bound: usize,
  offset: usize,
}

impl QueryIter {
  pub fn new(origin: *const (), size: usize) -> Result<Self> {
    // Do not use a buffered reader here to avoid multiple read(2) calls to the
    // proc file, ensuring a consistent snapshot of the virtual memory.
    let proc_maps = fs::read_to_string("/proc/self/maps").map_err(Error::SystemCall)?;

    Ok(Self {
      proc_maps,
      upper_bound: (origin as usize).saturating_add(size),
      offset: 0,
    })
  }

  pub fn upper_bound(&self) -> usize {
    self.upper_bound
  }
}

impl Iterator for QueryIter {
  type Item = Result<Region>;

  fn next(&mut self) -> Option<Self::Item> {
    let (line, _) = self.proc_maps.get(self.offset..)?.split_once('\n')?;
    self.offset += line.len() + 1;

    Some(parse_procfs_line(line).ok_or_else(|| Error::ProcfsInput(line.to_string())))
  }
}

/// Parses flags from /proc/[pid]/maps (e.g 'r--p').
fn parse_procfs_flags(protection: &str) -> (Protection, bool) {
  const MAPPINGS: &[Protection] = &[Protection::READ, Protection::WRITE, Protection::EXECUTE];

  let result = protection
    .chars()
    .zip(MAPPINGS.iter())
    .filter(|(c, _)| *c != '-')
    .fold(Protection::NONE, |acc, (_, prot)| acc | *prot);

  (result, protection.ends_with('s'))
}

/// Parses a line from /proc/[pid]/maps.
fn parse_procfs_line(input: &str) -> Option<Region> {
  let mut parts = input.split_whitespace();
  let mut memory = parts
    .next()?
    .split('-')
    .filter_map(|value| usize::from_str_radix(value, 16).ok());
  let (lower, upper) = (memory.next()?, memory.next()?);

  let flags = parts.next()?;
  let (protection, shared) = parse_procfs_flags(flags);

  Some(Region {
    base: lower as *const _,
    protection,
    shared,
    size: upper - lower,
    ..Region::default()
  })
}

#[cfg(test)]
mod tests {
  use super::{parse_procfs_flags, parse_procfs_line};
  use crate::Protection;

  #[test]
  fn procfs_flags_are_parsed() {
    let rwx = Protection::READ_WRITE_EXECUTE;

    assert_eq!(parse_procfs_flags("r--s"), (Protection::READ, true));
    assert_eq!(parse_procfs_flags("rw-p"), (Protection::READ_WRITE, false));
    assert_eq!(parse_procfs_flags("r-xs"), (Protection::READ_EXECUTE, true));
    assert_eq!(parse_procfs_flags("rwxs"), (rwx, true));
    assert_eq!(parse_procfs_flags("--xp"), (Protection::EXECUTE, false));
    assert_eq!(parse_procfs_flags("-w-s"), (Protection::WRITE, true));
  }

  #[test]
  fn procfs_regions_are_parsed() {
    let line = "00400000-00409000 r-xs 00000000 08:00 16088 /usr/bin/head";
    let region = parse_procfs_line(line).unwrap();

    assert_eq!(region.as_ptr(), 0x40_0000 as *mut ());
    assert_eq!(region.protection(), Protection::READ_EXECUTE);
    assert_eq!(region.len(), 0x9000);
    assert!(!region.is_guarded());
    assert!(region.is_shared());
  }
}
