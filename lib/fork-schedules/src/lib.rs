use beacon_api_types::{custom_types::Version, phase0::Fork};
use hex_literal::hex;
use macros::apply;
use unionlabs_primitives::FixedBytes;

#[derive(Debug)]
pub struct ForkSchedule {
    genesis_epoch: u64,
    genesis_version: Version,
    next_fork: Option<&'static ForkInfo>,
}

#[derive(Debug)]
pub struct ForkInfo {
    epoch: u64,
    version: Version,
    next_fork: Option<&'static ForkInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Forks {
    Altair = 1,
    Bellatrix = 2,
    Capella = 3,
    Deneb = 4,
    Electra = 5,
}

impl ForkSchedule {
    /// Create the fork schedule for the specified chain id. If the chain id is not well-known, [`LATEST`] will be returned.
    pub const fn for_chain_id(chain_id: u64) -> Self {
        match chain_id {
            MAINNET_CHAIN_ID => MAINNET,
            SEPOLIA_CHAIN_ID => SEPOLIA,
            HOLESKY_CHAIN_ID => HOLESKY,
            _ => LATEST,
        }
    }

    pub fn into_fork_schedule_list(self) -> Vec<Fork> {
        // this can probably be done more efficiently but whatever
        fn go(fs: &ForkInfo, previous_version: Version) -> Vec<Fork> {
            [Fork {
                current_version: fs.version,
                previous_version,
                epoch: fs.epoch,
            }]
            .into_iter()
            .chain(
                fs.next_fork
                    .map(|next| go(next, fs.version))
                    .into_iter()
                    .flatten(),
            )
            .collect()
        }

        go(
            &ForkInfo {
                epoch: self.genesis_epoch,
                version: self.genesis_version,
                next_fork: self.next_fork,
            },
            self.genesis_version,
        )
    }

    pub const fn genesis(&self) -> Fork {
        Fork {
            previous_version: self.genesis_version,
            current_version: self.genesis_version,
            epoch: self.genesis_epoch,
        }
    }

    pub const fn fork(&self, fork: Forks) -> Option<Fork> {
        const fn go(fs: &ForkInfo, previous_version: Version, depth: usize) -> Option<Fork> {
            if depth == 0 {
                Some(Fork {
                    previous_version,
                    current_version: fs.version,
                    epoch: fs.epoch,
                })
            } else {
                {
                    match fs.next_fork {
                        Some(n) => go(n, fs.version, depth - 1),
                        None => None,
                    }
                }
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
    ($(#[$attr:meta])* const $name:ident: _ = [($version:literal, $epoch:literal), $($tt:tt)*];) => {
        $(#[$attr])*
        pub const $name: ForkSchedule = ForkSchedule {
            genesis_epoch: $epoch,
            genesis_version: Version(FixedBytes::new(hex!($version))),
            next_fork: fork_schedule!($($tt)*),
        };
    };
    (($version:literal, $epoch:literal), $($tt:tt)*) => {
        Some(&ForkInfo {
            epoch: $epoch,
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
const MAINNET: _ = [
    ("00000000", 0),
    ("01000000", 74240),
    ("02000000", 144896),
    ("03000000", 194048),
    ("04000000", 269568),
];

pub const MAINNET_CHAIN_ID: u64 = 1;

/// The fork schedule for Sepolia (chain id `11155111`).
///
/// <https://ethereum-sepolia-beacon-api.publicnode.com/eth/v1/config/fork_schedule>
#[apply(fork_schedule)]
const SEPOLIA: _ = [
    ("90000069", 0),
    ("90000070", 50),
    ("90000071", 100),
    ("90000072", 56832),
    ("90000073", 132608),
    ("90000074", 222464),
];

pub const SEPOLIA_CHAIN_ID: u64 = 11155111;

/// The fork schedule for Holešky (chain id `17000`).
///
/// <https://ethereum-holesky-beacon-api.publicnode.com/eth/v1/config/fork_schedule>
#[apply(fork_schedule)]
const HOLESKY: _ = [
    ("01017000", 0),
    ("02017000", 0),
    ("03017000", 0),
    ("04017000", 256),
    ("05017000", 29696),
    ("06017000", 115968),
];

pub const HOLESKY_CHAIN_ID: u64 = 17000;

/// The fork schedule for a chain starting on the latest fork (currently electra).
///
/// This is mainly used for devnets.
#[apply(fork_schedule)]
const LATEST: _ = [
    ("00000001", 0),
    ("01000001", 0),
    ("02000001", 0),
    ("03000001", 0),
    ("04000001", 0),
    ("04000001", 0),
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
                    epoch: 0,
                },
                Fork {
                    previous_version: Version(hex!("00000000").into()),
                    current_version: Version(hex!("01000000").into()),
                    epoch: 74240,
                },
                Fork {
                    previous_version: Version(hex!("01000000").into()),
                    current_version: Version(hex!("02000000").into()),
                    epoch: 144896,
                },
                Fork {
                    previous_version: Version(hex!("02000000").into()),
                    current_version: Version(hex!("03000000").into()),
                    epoch: 194048,
                },
                Fork {
                    previous_version: Version(hex!("03000000").into()),
                    current_version: Version(hex!("04000000").into()),
                    epoch: 269568,
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
                epoch: 0,
            },
        );

        assert_eq!(
            MAINNET.fork(Forks::Altair),
            Some(Fork {
                previous_version: Version(hex!("00000000").into()),
                current_version: Version(hex!("01000000").into()),
                epoch: 74240,
            }),
        );

        assert_eq!(
            MAINNET.fork(Forks::Bellatrix),
            Some(Fork {
                previous_version: Version(hex!("01000000").into()),
                current_version: Version(hex!("02000000").into()),
                epoch: 144896,
            }),
        );

        assert_eq!(MAINNET.fork(Forks::Electra), None);
    }
}
