//! Fork schedules for well-known beacon chains.
//!
//! <https://github.com/ethereum/consensus-specs>

#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::unreadable_literal)]

use beacon_api_types::{
    custom_types::{Epoch, Version},
    phase0::Fork,
};
use hex_literal::hex;
use macros::apply;
use unionlabs_primitives::FixedBytes;

/// The fork schedule for an ethereum beacon chain.
#[derive(Debug)]
pub struct ForkSchedule {
    genesis_epoch: Epoch,
    genesis_version: Version,
    next_fork: Option<&'static ForkInfo>,
}

#[derive(Debug)]
struct ForkInfo {
    epoch: Epoch,
    version: Version,
    next_fork: Option<&'static ForkInfo>,
}

/// Known beacon chain forks.
///
/// <https://github.com/ethereum/consensus-specs/tree/dev/specs>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Forks {
    /// <https://github.com/ethereum/consensus-specs/tree/dev/specs/altair>
    Altair = 1,
    /// <https://github.com/ethereum/consensus-specs/tree/dev/specs/bellatrix>
    Bellatrix = 2,
    /// <https://github.com/ethereum/consensus-specs/tree/dev/specs/capella>
    Capella = 3,
    /// <https://github.com/ethereum/consensus-specs/tree/dev/specs/deneb>
    Deneb = 4,
    /// <https://github.com/ethereum/consensus-specs/tree/dev/specs/electra>
    Electra = 5,
}

impl ForkSchedule {
    /// Create the fork schedule for the specified chain id. If the chain id is not well-known, [`LATEST`] will be returned.
    #[must_use]
    pub const fn for_chain_id(chain_id: u64) -> Self {
        match chain_id {
            MAINNET_CHAIN_ID => MAINNET,
            SEPOLIA_CHAIN_ID => SEPOLIA,
            HOLESKY_CHAIN_ID => HOLESKY,
            _ => LATEST,
        }
    }

    /// Get the [`Fork`] list for this fork schedule.
    ///
    /// This follows the same format as defined in the [`/eth/v1/config/fork_schedule`] endpoint.
    ///
    /// ```rust
    /// # use fork_schedules::MAINNET;
    /// # use hex_literal::hex;
    /// # use beacon_api_types::{custom_types::{Version, Epoch}, phase0::Fork};
    /// assert_eq!(
    ///     MAINNET.into_fork_schedule_list()[3],
    ///     Fork {
    ///         previous_version: Version(hex!("02000000").into()),
    ///         current_version: Version(hex!("03000000").into()),
    ///         epoch: Epoch::new(194048),
    ///     }
    /// );
    /// ```
    ///
    /// [`/eth/v1/config/fork_schedule`]: https://ethereum.github.io/beacon-APIs/?urls.primaryName=v1#/Config/getForkSchedule
    #[must_use]
    pub fn into_fork_schedule_list(self) -> Vec<Fork> {
        fn go(fork_info: &ForkInfo, previous_version: Version, out: &mut Vec<Fork>) {
            out.push(Fork {
                current_version: fork_info.version,
                previous_version,
                epoch: fork_info.epoch,
            });

            if let Some(next) = fork_info.next_fork {
                go(next, fork_info.version, out);
            }
        }

        let mut out = Vec::with_capacity(self.len());

        go(
            &ForkInfo {
                epoch: self.genesis_epoch,
                version: self.genesis_version,
                next_fork: self.next_fork,
            },
            self.genesis_version,
            &mut out,
        );

        debug_assert!(out.len() == self.len());

        out
    }

    const fn len(&self) -> usize {
        const fn go(fork_info: &ForkInfo) -> usize {
            1 + match fork_info.next_fork {
                Some(x) => go(x),
                None => 0,
            }
        }

        1 + match self.next_fork {
            Some(x) => go(x),
            None => 0,
        }
    }

    /// Get the genesis [`Fork`] for this fork schedule.
    ///
    /// ```rust
    /// # use fork_schedules::{SEPOLIA, Forks};
    /// # use hex_literal::hex;
    /// # use beacon_api_types::{custom_types::{Version, Epoch}, phase0::Fork};
    /// assert_eq!(
    ///     SEPOLIA.genesis(),
    ///     Fork {
    ///         previous_version: Version(hex!("90000069").into()),
    ///         current_version: Version(hex!("90000069").into()),
    ///         epoch: Epoch::new(0)
    ///     }
    /// );
    /// ```
    #[must_use]
    pub const fn genesis(&self) -> Fork {
        Fork {
            previous_version: self.genesis_version,
            current_version: self.genesis_version,
            epoch: self.genesis_epoch,
        }
    }

    /// Get the [`Fork`] associated with the [`Forks`] for this fork schedule.
    ///
    /// ```rust
    /// # use fork_schedules::{HOLESKY, Forks};
    /// # use hex_literal::hex;
    /// # use beacon_api_types::{custom_types::{Version, Epoch}, phase0::Fork};
    /// assert_eq!(
    ///     HOLESKY.fork(Forks::Capella),
    ///     Some(Fork {
    ///         previous_version: Version(hex!("03017000").into()),
    ///         current_version: Version(hex!("04017000").into()),
    ///         epoch: Epoch::new(256),
    ///     })
    /// );
    /// ```
    ///
    /// Note that this will return `None` if the fork schedule has not yet been updated to the specified fork:
    ///
    /// ```rust
    /// # use fork_schedules::{MAINNET, Forks};
    /// # use hex_literal::hex;
    /// # use beacon_api_types::custom_types::Version;
    /// assert_eq!(
    ///     MAINNET.fork(Forks::Electra),
    ///     None
    /// );
    /// ```
    #[must_use]
    pub const fn fork(&self, fork: Forks) -> Option<Fork> {
        const fn go(fs: &ForkInfo, previous_version: Version, depth: usize) -> Option<Fork> {
            if depth == 0 {
                Some(Fork {
                    previous_version,
                    current_version: fs.version,
                    epoch: fs.epoch,
                })
            } else if let Some(n) = fs.next_fork {
                go(n, fs.version, depth - 1)
            } else {
                None
            }
        }

        go(
            &ForkInfo {
                epoch: self.genesis_epoch,
                version: self.genesis_version,
                next_fork: self.next_fork,
            },
            self.genesis_version,
            fork as usize,
        )
    }
}

macro_rules! fork_schedule {
    ($(#[$attr:meta])* pub const $name:ident: _ = [($version:literal, $epoch:literal), $($tt:tt)*];) => {
        $(#[$attr])*
        pub const $name: ForkSchedule = ForkSchedule {
            genesis_epoch: Epoch::new($epoch),
            genesis_version: Version(FixedBytes::new(hex!($version))),
            next_fork: fork_schedule!($($tt)*),
        };
    };
    (($version:literal, $epoch:literal), $($tt:tt)*) => {
        Some(&ForkInfo {
            epoch: Epoch::new($epoch),
            version: Version(FixedBytes::new(hex!($version))),
            next_fork: fork_schedule!($($tt)*),
        })
    };
    () => {
        None
    };
}

/// The fork schedule for Mainnet (chain id `1`).
///
/// <https://ethereum-beacon-api.publicnode.com/eth/v1/config/fork_schedule>
#[apply(fork_schedule)]
pub const MAINNET: _ = [
    ("00000000", 0),      // phase0
    ("01000000", 74240),  // altair
    ("02000000", 144896), // bellatrix
    ("03000000", 194048), // capella
    ("04000000", 269568), // deneb
];

/// Mainnet chain id.
///
/// <https://chainlist.org/chain/1>
pub const MAINNET_CHAIN_ID: u64 = 1;

/// The fork schedule for Sepolia (chain id `11155111`).
///
/// <https://ethereum-sepolia-beacon-api.publicnode.com/eth/v1/config/fork_schedule>
#[apply(fork_schedule)]
pub const SEPOLIA: _ = [
    ("90000069", 0),      // phase0
    ("90000070", 50),     // altair
    ("90000071", 100),    // bellatrix
    ("90000072", 56832),  // capella
    ("90000073", 132608), // deneb
    ("90000074", 222464), // electra
];

/// Sepolia chain id.
///
/// <https://chainlist.org/chain/11155111>
pub const SEPOLIA_CHAIN_ID: u64 = 11155111;

/// The fork schedule for Holešky (chain id `17000`).
///
/// <https://ethereum-holesky-beacon-api.publicnode.com/eth/v1/config/fork_schedule>
#[apply(fork_schedule)]
pub const HOLESKY: _ = [
    ("01017000", 0),      // phase0
    ("02017000", 0),      // altair
    ("03017000", 0),      // bellatrix
    ("04017000", 256),    // capella
    ("05017000", 29696),  // deneb
    ("06017000", 115968), // electra
];

/// Holesky chain id.
///
/// <https://chainlist.org/chain/17000>
pub const HOLESKY_CHAIN_ID: u64 = 17000;

/// The fork schedule for a chain starting on the latest fork (currently electra).
///
/// This is mainly used for devnets.
#[apply(fork_schedule)]
pub const LATEST: _ = [
    ("00000001", 0), // phase0
    ("01000001", 0), // altair
    ("02000001", 0), // bellatrix
    ("03000001", 0), // capella
    ("04000001", 0), // deneb
                     // ("04000001", 0), // electra
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_fork_schedule_list() {
        assert_eq!(
            MAINNET.into_fork_schedule_list(),
            vec![
                Fork {
                    previous_version: Version(hex!("00000000").into()),
                    current_version: Version(hex!("00000000").into()),
                    epoch: Epoch::new(0),
                },
                Fork {
                    previous_version: Version(hex!("00000000").into()),
                    current_version: Version(hex!("01000000").into()),
                    epoch: Epoch::new(74240),
                },
                Fork {
                    previous_version: Version(hex!("01000000").into()),
                    current_version: Version(hex!("02000000").into()),
                    epoch: Epoch::new(144896),
                },
                Fork {
                    previous_version: Version(hex!("02000000").into()),
                    current_version: Version(hex!("03000000").into()),
                    epoch: Epoch::new(194048),
                },
                Fork {
                    previous_version: Version(hex!("03000000").into()),
                    current_version: Version(hex!("04000000").into()),
                    epoch: Epoch::new(269568),
                },
            ]
        );
    }

    #[test]
    fn get_fork() {
        assert_eq!(
            MAINNET.genesis(),
            Fork {
                previous_version: Version(hex!("00000000").into()),
                current_version: Version(hex!("00000000").into()),
                epoch: Epoch::new(0),
            },
        );

        assert_eq!(
            MAINNET.fork(Forks::Altair),
            Some(Fork {
                previous_version: Version(hex!("00000000").into()),
                current_version: Version(hex!("01000000").into()),
                epoch: Epoch::new(74240),
            }),
        );

        assert_eq!(
            MAINNET.fork(Forks::Bellatrix),
            Some(Fork {
                previous_version: Version(hex!("01000000").into()),
                current_version: Version(hex!("02000000").into()),
                epoch: Epoch::new(144896),
            }),
        );

        assert_eq!(MAINNET.fork(Forks::Electra), None);
    }

    #[test]
    fn test_len() {
        assert_eq!(MAINNET.len(), 5);
    }
}
