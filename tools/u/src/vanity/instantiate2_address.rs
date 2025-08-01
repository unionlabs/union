use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use alloy::{hex, primitives::U256};
use anyhow::bail;
use clap::Args;
use sha2::Digest;
use unionlabs::primitives::{Bech32, Bytes, H256};

#[derive(Debug, Args)]
pub struct Cmd {
    pub creator: Bech32<Bytes>,
    pub prefix: String,
    // pub suffix: String,
    /// Number of threads to use for parallel processing
    #[arg(long, default_value_t = num_cpus::get())]
    threads: usize,
}

impl Cmd {
    pub fn run(&self) -> anyhow::Result<()> {
        if self
            .prefix
            .bytes()
            .any(|b| !subtle_encoding::bech32::DEFAULT_CHARSET.contains(&(b as char)))
        {
            bail!("invalid prefix");
        }

        // if self
        //     .suffix
        //     .bytes()
        //     .any(|b| !subtle_encoding::bech32::DEFAULT_CHARSET.contains(&(b as char)))
        // {
        //     bail!("invalid suffix");
        // }

        let found = Arc::new(AtomicBool::new(false));
        let total_attempts = Arc::new(AtomicU64::new(0));
        let start_time = Instant::now();

        let preimage = vec![
            // sha256("module")
            hex!("120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9").as_slice(),
            b"wasm\0",
            &const { 32_u64.to_be_bytes() },
            // bytecode base checksum
            &hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
            &(self.creator.data().len() as u64).to_be_bytes(),
            &self.creator.data(),
            &const { 32_u64.to_be_bytes() },
            &[0; 32],
            &const { 0_u64.to_be_bytes() },
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

        let mut handles = Vec::new();
        for i in 0..self.threads {
            let found = Arc::clone(&found);
            let total_attempts = Arc::clone(&total_attempts);
            let prefix = self.prefix.clone();
            // let suffix = self.suffix.clone();
            let creator = self.creator.clone();

            let mut preimage = preimage.clone();
            let range = (preimage.len() - 8 - 32)..(preimage.len() - 8);

            let handle = thread::spawn(move || -> Option<H256> {
                let mut local_attempts = 0u64;

                let mut salt = U256::from(u64::MAX) * U256::from(i);

                while !found.load(Ordering::Relaxed) {
                    loop {
                        local_attempts += 1;
                        preimage[range.clone()].copy_from_slice(&salt.to_be_bytes::<32>());
                        let res: H256 = sha2::Sha256::digest(&preimage).into();

                        let addr = subtle_encoding::bech32::encode(creator.hrp(), res);

                        if addr[creator.hrp().len() + 1..].starts_with(&prefix)
                        // && addr.ends_with(&suffix)
                        {
                            let salt_bytes = H256::new(salt.to_be_bytes());

                            println!("{addr}");
                            println!("{}", salt_bytes);
                            found.store(true, Ordering::Relaxed);
                            total_attempts.fetch_add(local_attempts, Ordering::Relaxed);
                            return Some(salt_bytes);
                        }

                        salt += U256::ONE;

                        if local_attempts % 20000 == 0 {
                            total_attempts.fetch_add(20000, Ordering::Relaxed);
                            local_attempts = 0;
                            break;
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

        if let Some(salt_bytes) = result {
            let final_attempts = total_attempts.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed();
            println!(
                "Found vanity address after {} attempts in {:.2}s!",
                final_attempts,
                elapsed.as_secs_f64()
            );
            println!("Salt: {}", salt_bytes);
            println!(
                "Contract address: {}",
                Bech32::new(
                    &self.creator.hrp(),
                    cosmwasm_std::instantiate2_address(
                        // bytecode base checksum
                        &hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1"),
                        &self.creator.data().to_vec().into(),
                        salt_bytes.as_ref(),
                    )?
                    .to_vec()
                )
            );
        } else {
            println!("Search was interrupted before finding a match.");
        }

        Ok(())
    }
}
