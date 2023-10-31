//! Global override of color control

#![cfg_attr(not(test), no_std)]

use core::sync::atomic::{AtomicUsize, Ordering};

/// Selection for overriding color output
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorChoice {
    Auto,
    AlwaysAnsi,
    Always,
    Never,
}

impl ColorChoice {
    /// Get the current [`ColorChoice`] state
    pub fn global() -> Self {
        USER.get()
    }

    /// Override the detected [`ColorChoice`]
    pub fn write_global(self) {
        USER.set(self)
    }
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Auto
    }
}

static USER: AtomicChoice = AtomicChoice::new();

#[derive(Debug)]
pub(crate) struct AtomicChoice(AtomicUsize);

impl AtomicChoice {
    pub(crate) const fn new() -> Self {
        Self(AtomicUsize::new(Self::from_choice(
            crate::ColorChoice::Auto,
        )))
    }

    pub(crate) fn get(&self) -> crate::ColorChoice {
        let choice = self.0.load(Ordering::SeqCst);
        Self::to_choice(choice).unwrap()
    }

    pub(crate) fn set(&self, choice: crate::ColorChoice) {
        let choice = Self::from_choice(choice);
        self.0.store(choice, Ordering::SeqCst);
    }

    const fn from_choice(choice: crate::ColorChoice) -> usize {
        match choice {
            crate::ColorChoice::Auto => 0,
            crate::ColorChoice::AlwaysAnsi => 1,
            crate::ColorChoice::Always => 2,
            crate::ColorChoice::Never => 3,
        }
    }

    const fn to_choice(choice: usize) -> Option<crate::ColorChoice> {
        match choice {
            0 => Some(crate::ColorChoice::Auto),
            1 => Some(crate::ColorChoice::AlwaysAnsi),
            2 => Some(crate::ColorChoice::Always),
            3 => Some(crate::ColorChoice::Never),
            _ => None,
        }
    }
}

impl Default for AtomicChoice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn choice_serialization() {
        let expected = vec![
            ColorChoice::Auto,
            ColorChoice::AlwaysAnsi,
            ColorChoice::Always,
            ColorChoice::Never,
        ];
        let values: Vec<_> = expected
            .iter()
            .cloned()
            .map(AtomicChoice::from_choice)
            .collect();
        let actual: Vec<_> = values
            .iter()
            .cloned()
            .filter_map(AtomicChoice::to_choice)
            .collect();
        assert_eq!(expected, actual);
    }
}
