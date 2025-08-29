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
    hex,
    primitives::{keccak256, Address, U256},
};
use clap::Args;
use sha2::digest::{generic_array::GenericArray, FixedOutputReset, Update};
use unionlabs::{
    primitives::{ByteArrayExt, H160, H256},
    typenum,
};

#[derive(Debug, Args)]
pub struct Cmd {
    pub deployer: Address,
    pub sender: Address,
    #[arg(long)]
    pub prefix: Option<String>,
    #[arg(long)]
    pub suffix: Option<String>,
    /// Number of threads to use for parallel processing
    #[arg(long, default_value_t = num_cpus::get())]
    pub threads: usize,
    #[arg(long, default_value_t = U256::ZERO)]
    pub seed: U256,
}

impl Cmd {
    pub fn run(&self) -> anyhow::Result<()> {
        let found = Arc::new(AtomicBool::new(false));
        let total_attempts = Arc::new(AtomicU64::new(0));
        let start_time = Instant::now();

        let (prefix_bytes, prefix_nibble) = self
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
                    let nibble = u8::from_str_radix(&nibble_char.to_string(), 16)
                        .expect("Invalid hex nibble");
                    (
                        hex::decode(full_bytes).expect("Invalid hex prefix"),
                        Some(nibble),
                    )
                }
            })
            .unwrap_or((Vec::new(), None));

        let (suffix_bytes, suffix_leading_nibble) = self
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

        let mut salt_preimage: [u8; 42 + 1 + 32] = (<H160>::new(self.sender.into()).to_string()
            + "/")
            .into_bytes()
            .into_iter()
            .chain([0; 32])
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let seed = self.seed;
        let range = (salt_preimage.len() - 32)..salt_preimage.len();

        let mut handles = Vec::new();
        for i in 0..self.threads {
            let found = Arc::clone(&found);
            let total_attempts = Arc::clone(&total_attempts);
            let deployer = self.deployer;
            let prefix_bytes = prefix_bytes.clone();
            let suffix_bytes = suffix_bytes.clone();

            let handle = thread::spawn(move || -> Option<H256> {
                let mut local_attempts = 0u64;

                let mut salt = (0..(i + 1)).fold(seed, |acc, _| {
                    U256::from_be_bytes(
                        <sha2::Sha256 as sha2::Digest>::digest(acc.to_be_bytes::<32>()).into(),
                    )
                });
                println!("{i}: {salt}");

                *salt_preimage.array_slice_mut::<{ 42 + 1 }, 32>() = salt.to_be_bytes();

                let mut counter =
                    u64::from_be_bytes(salt.to_be_bytes::<32>().array_slice::<0, 8>());

                let mut proxy_preimage: [u8; 1 + 20 + 32 + 32] = [0xff]
                    .into_iter()
                    .chain(deployer)
                    .chain(salt.to_be_bytes::<32>())
                    .chain(hex!(
                        "21c35dbe1b344a2488cf3321d6ce542f8e9f305544ff09e4993a62319a497c1f"
                    ))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();

                let mut hasher = <keccak_asm::Keccak256 as sha2::Digest>::new();

                let mut out: GenericArray<u8, typenum::U32> = [0_u8; 32].into();

                while !found.load(Ordering::Relaxed) {
                    'inner: while local_attempts < 100000 {
                        *salt_preimage.array_slice_mut::<{ 42 + 1 }, 8>() = counter.to_be_bytes();

                        <_ as Update>::update(&mut hasher, &salt_preimage);
                        <_ as FixedOutputReset>::finalize_into_reset(
                            &mut hasher,
                            proxy_preimage.array_slice_mut::<21, 32>().into(),
                        );

                        <_ as Update>::update(&mut hasher, &proxy_preimage);
                        <_ as FixedOutputReset>::finalize_into_reset(&mut hasher, &mut out);

                        <_ as Update>::update(&mut hasher, &[0xd6, 0x94]);
                        <_ as Update>::update(&mut hasher, &out[12..]);
                        <_ as Update>::update(&mut hasher, &[0x01]);
                        <_ as FixedOutputReset>::finalize_into_reset(&mut hasher, &mut out);

                        let addr_bytes = &out[12..];

                        let matches_prefix = {
                            if let Some(nibble) = prefix_nibble {
                                let bytes_match = addr_bytes[..prefix_bytes.len()] == prefix_bytes;
                                bytes_match
                                    && ((addr_bytes[prefix_bytes.len()] >> 4) & 0xF == nibble)
                            } else {
                                addr_bytes[..prefix_bytes.len()] == prefix_bytes
                            }
                        };

                        let matches_suffix = || {
                            if let Some(leading_nibble) = suffix_leading_nibble {
                                let bytes_match =
                                    addr_bytes[20 - suffix_bytes.len()..] == suffix_bytes;

                                bytes_match
                                    && (addr_bytes[20 - suffix_bytes.len() - 1] & 0xF
                                        == leading_nibble)
                            } else {
                                addr_bytes[20 - suffix_bytes.len()..] == suffix_bytes
                            }
                        };

                        if matches_prefix && matches_suffix() {
                            let salt_bytes =
                                H256::new(salt_preimage.array_slice::<{ 42 + 1 }, 32>());
                            found.store(true, Ordering::Relaxed);
                            total_attempts.fetch_add(local_attempts, Ordering::Relaxed);
                            println!("Salt: {}", salt_bytes);
                            println!("Address: {}", <H160>::try_from(addr_bytes).unwrap());
                            return Some(salt_bytes);
                        } else {
                            counter = counter.wrapping_add(1);
                            local_attempts += 1;

                            continue 'inner;
                        }
                    }

                    total_attempts.fetch_add(local_attempts, Ordering::Relaxed);
                    local_attempts = 0;
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

        if let Some(salt_bytes) = result {
            let final_attempts = total_attempts.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed();
            println!(
                "Found vanity address after {} attempts in {:.2}s!",
                final_attempts,
                elapsed.as_secs_f64()
            );
            println!("Salt: {}", salt_bytes);
            salt_preimage[range.clone()].copy_from_slice(salt_bytes.get());
            println!(
                "Contract address: {}",
                ::create3::predict_deterministic_address(self.deployer, keccak256(salt_preimage))
            );
        } else {
            println!("Search was interrupted before finding a match.");
        }

        Ok(())
    }
}
