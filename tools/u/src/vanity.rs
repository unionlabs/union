use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use alloy::{
    primitives::{Address, Bytes, B256, U256},
    sol,
    sol_types::{SolCall, SolValue},
};
use anyhow::Result;
use clap::{Args, Subcommand};
use rand::RngCore;
use unionlabs::ethereum::keccak256;

sol! {
    struct FungibleAssetMetadata {
        bytes implementation;
        bytes initializer;
    }

    function initialize(
        address _authority,
        address _minter,
        string calldata _name,
        string calldata _symbol,
        uint8 _decimals,
        bytes calldata _salt
    ) external;
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    #[command(visible_alias = "wt")]
    WrappedToken(WrappedTokenArgs),
}

#[derive(Debug, Args)]
pub struct WrappedTokenArgs {
    /// Forward path (uint256) for the wrapped token
    #[arg(long)]
    path: u64,

    /// Channel ID for the wrapped token
    #[arg(long)]
    channel_id: u32,

    /// Base token bytes (hex encoded)
    #[arg(long)]
    token: unionlabs::primitives::Bytes,

    /// Address prefix to match (hex encoded, e.g., "0x1234")
    #[arg(long)]
    prefix: Option<String>,

    /// Address suffix to match (hex encoded, e.g., "5678")
    #[arg(long)]
    suffix: Option<String>,

    /// Manager/Authority address for U.initialize
    #[arg(long)]
    auth: unionlabs::primitives::FixedBytes<20>,

    /// Zkgm (minter) address for U.initialize
    #[arg(long)]
    zkgm: unionlabs::primitives::FixedBytes<20>,

    /// U implementation address
    #[arg(long)]
    impl_address: unionlabs::primitives::FixedBytes<20>,

    /// Token name (default: "Union")
    #[arg(long, default_value = "Union")]
    name: String,

    /// Token symbol (default: "U")
    #[arg(long, default_value = "U")]
    symbol: String,

    /// Token decimals (default: 18)
    #[arg(long, default_value = "18")]
    decimals: u8,

    /// Number of threads to use for parallel processing
    #[arg(long, default_value_t = num_cpus::get())]
    threads: usize,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match self {
            Cmd::WrappedToken(args) => find_vanity_wrapped_token(args).await,
        }
    }
}

async fn find_vanity_wrapped_token(args: WrappedTokenArgs) -> Result<()> {
    let token_bytes = args.token.into_vec();
    let auth_address: Address = args.auth.get().into();
    let zkgm_address: Address = args.zkgm.get().into();
    let impl_address: Address = args.impl_address.get().into();

    let (prefix_bytes, prefix_nibble) = args
        .prefix
        .as_ref()
        .map(|p| {
            let hex_str = if p.starts_with("0x") || p.starts_with("0X") {
                &p[2..]
            } else {
                p
            };

            if hex_str.len() % 2 == 0 {
                // Even length - full bytes
                (hex::decode(hex_str).expect("Invalid hex prefix"), None)
            } else {
                // Odd length - last character is a nibble constraint
                let full_bytes = &hex_str[..hex_str.len() - 1];
                let nibble_char = hex_str.chars().last().unwrap();
                let nibble =
                    u8::from_str_radix(&nibble_char.to_string(), 16).expect("Invalid hex nibble");
                (
                    hex::decode(full_bytes).expect("Invalid hex prefix"),
                    Some(nibble),
                )
            }
        })
        .unwrap_or((Vec::new(), None));

    let (suffix_bytes, suffix_leading_nibble) = args
        .suffix
        .as_ref()
        .map(|s| {
            let hex_str = if s.starts_with("0x") || s.starts_with("0X") {
                &s[2..]
            } else {
                s
            };

            if hex_str.len() % 2 == 0 {
                // Even length - full bytes
                (hex::decode(hex_str).expect("Invalid hex suffix"), None)
            } else {
                // Odd length - address must end with the literal hex pattern
                let leading_nibble =
                    u8::from_str_radix(&hex_str[0..1], 16).expect("Invalid hex nibble");
                let remaining_bytes = hex::decode(&hex_str[1..]).expect("Invalid hex suffix");
                (remaining_bytes, Some(leading_nibble))
            }
        })
        .unwrap_or((Vec::new(), None));

    println!("Searching for vanity address...");
    println!("Path: {}", args.path);
    println!("Channel ID: {}", args.channel_id);
    println!("Token: 0x{}", hex::encode(&token_bytes));
    println!("Auth: {}", auth_address);
    println!("Zkgm: {}", zkgm_address);
    println!("Implementation: {}", impl_address);
    if let Some(ref p) = args.prefix {
        println!("Prefix: {}", p);
    }
    if let Some(ref s) = args.suffix {
        println!("Suffix: {}", s);
    }
    println!("Threads: {}", args.threads);
    println!();

    let found = Arc::new(AtomicBool::new(false));
    let total_attempts = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();

    let mut handles = Vec::new();
    for _ in 0..args.threads {
        let found = Arc::clone(&found);
        let total_attempts = Arc::clone(&total_attempts);
        let prefix_bytes = prefix_bytes.clone();
        let suffix_bytes = suffix_bytes.clone();
        let name = args.name.clone();
        let symbol = args.symbol.clone();
        let token_bytes = token_bytes.clone();

        let handle = thread::spawn(move || -> Option<(Vec<u8>, Address)> {
            let mut rng = rand::thread_rng();
            let mut local_attempts = 0u64;

            let mut preimage = FungibleAssetMetadata {
                implementation: Bytes::from(impl_address.to_vec()),
                initializer: Bytes::from(
                    initializeCall {
                        _authority: auth_address,
                        _minter: zkgm_address,
                        _name: name.clone(),
                        _symbol: symbol.clone(),
                        _decimals: args.decimals,
                        _salt: Bytes::from(vec![0xFFu8; 20]),
                    }
                    .abi_encode(),
                ),
            }
            .abi_encode_params();
            let preimage_salt_index = preimage.len() - 20 - 40;

            while !found.load(Ordering::Relaxed) {
                local_attempts += 1;
                {
                    rng.fill_bytes(&mut preimage[preimage_salt_index..preimage_salt_index + 20]);
                }
                {
                    let metadata_image = keccak256(&preimage);
                    let wrapped_token_salt = B256::from_slice(
                        keccak256(
                            (
                                U256::from(args.path),
                                U256::from(args.channel_id),
                                Bytes::from(token_bytes.clone()),
                                metadata_image,
                            )
                                .abi_encode_params(),
                        )
                        .get(),
                    );

                    let wrapped_token =
                        create3::predict_deterministic_address(zkgm_address, wrapped_token_salt);

                    let address_bytes = wrapped_token.as_slice();

                    let matches_prefix = if prefix_bytes.is_empty() && prefix_nibble.is_none() {
                        true
                    } else {
                        let full_bytes_match = if prefix_bytes.is_empty() {
                            true
                        } else if address_bytes.len() < prefix_bytes.len() {
                            false
                        } else {
                            address_bytes[..prefix_bytes.len()] == prefix_bytes[..]
                        };

                        if let Some(nibble) = prefix_nibble {
                            if address_bytes.len() <= prefix_bytes.len() {
                                false
                            } else {
                                let byte_to_check = address_bytes[prefix_bytes.len()];
                                let high_nibble = (byte_to_check >> 4) & 0xF;
                                full_bytes_match && high_nibble == nibble
                            }
                        } else {
                            full_bytes_match
                        }
                    };

                    let matches_suffix = if suffix_bytes.is_empty()
                        && suffix_leading_nibble.is_none()
                    {
                        true
                    } else if let Some(leading_nibble) = suffix_leading_nibble {
                        let required_bytes = suffix_bytes.len() + 1; // +1 for the nibble
                        if address_bytes.len() < required_bytes {
                            false
                        } else {
                            let suffix_start = address_bytes.len() - suffix_bytes.len();
                            let bytes_match = if suffix_bytes.is_empty() {
                                true
                            } else {
                                address_bytes[suffix_start..] == suffix_bytes[..]
                            };

                            let nibble_byte_index = address_bytes.len() - suffix_bytes.len() - 1;
                            let byte_with_nibble = address_bytes[nibble_byte_index];
                            let low_nibble = byte_with_nibble & 0xF;
                            let nibble_matches = low_nibble == leading_nibble;

                            bytes_match && nibble_matches
                        }
                    } else if address_bytes.len() < suffix_bytes.len() {
                        false
                    } else {
                        let suffix_start = address_bytes.len() - suffix_bytes.len();
                        address_bytes[suffix_start..] == suffix_bytes[..]
                    };

                    if matches_prefix && matches_suffix {
                        found.store(true, Ordering::Relaxed);
                        total_attempts.fetch_add(local_attempts, Ordering::Relaxed);
                        return Some((
                            preimage[preimage_salt_index..preimage_salt_index + 20].to_vec(),
                            wrapped_token,
                        ));
                    }

                    if local_attempts % 20000 == 0 {
                        total_attempts.fetch_add(20000, Ordering::Relaxed);
                        local_attempts = 0;
                    }
                }
            }

            total_attempts.fetch_add(local_attempts, Ordering::Relaxed);
            None
        });

        handles.push(handle);
    }

    let found_reporter = Arc::clone(&found);
    let total_attempts_reporter = Arc::clone(&total_attempts);
    let status_handle = thread::spawn(move || {
        while !found_reporter.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_secs(1));
            let attempts = total_attempts_reporter.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed();
            let rate = attempts as f64 / elapsed.as_secs_f64();
            print!("\rAttempts: {} ({:.0} attempts/sec)...", attempts, rate);
            let _ = std::io::stdout().flush();
        }
    });

    let mut result = None;
    for handle in handles {
        if let Ok(thread_result) = handle.join() {
            if thread_result.is_some() {
                result = thread_result;
                break;
            }
        }
    }

    status_handle.join().ok();

    println!();

    if let Some((salt, wrapped_token)) = result {
        let final_attempts = total_attempts.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed();
        println!(
            "Found vanity address after {} attempts in {:.2}s!",
            final_attempts,
            elapsed.as_secs_f64()
        );
        println!("Salt: 0x{}", hex::encode(&salt));
        println!("Wrapped Token Address: {:#x}", wrapped_token);
    } else {
        println!("Search was interrupted before finding a match.");
    }

    Ok(())
}
