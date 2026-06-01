use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use gno_rpc::types::ValidatorSet;
use unionlabs::{
    bounded::BoundedI64,
    ibc::core::client::height::Height,
    primitives::{Bytes, encoding::Base64},
};

use crate::print_json;

#[derive(Debug, Args)]
pub struct Cmd {
    #[arg(global = true, short = 'r', default_value = "http://localhost:26657")]
    pub rpc_url: String,
    #[command(subcommand)]
    pub cmd: SubCmd,
}

#[derive(Debug, Subcommand)]
pub enum SubCmd {
    /// Fetch the light client update header for the specified heights.
    FetchHeader {
        from: BoundedI64<0>,
        to: BoundedI64<0>,
    },
    /// Fetch the light client update header for the specified heights.
    FetchProof {
        path: String,
        data: Bytes<Base64>,
        height: BoundedI64<1>,
    },
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let client = gno_rpc::Client::new(self.rpc_url).await?;

        match self.cmd {
            SubCmd::FetchHeader { from, to } => {
                if from >= to {
                    bail!("from must be < to")
                }

                let trusted_commit = client.commit(from).await?;

                let untrusted_commit = client.commit(to).await?;

                let trusted_validators = client.validators(from).await?.validators;

                let untrusted_validators = client.validators(to).await?.validators;

                let header = gno_light_client_types::Header {
                    validator_set: ValidatorSet {
                        proposer: untrusted_validators
                            .iter()
                            .find(|val| {
                                val.address
                                    == untrusted_commit.signed_header.header.proposer_address
                            })
                            .expect("proposer must exist in set")
                            .clone(),
                        validators: untrusted_validators,
                    },
                    trusted_height: Height::new(from.inner() as u64),
                    trusted_validators: ValidatorSet {
                        proposer: trusted_validators
                            .iter()
                            .find(|val| {
                                val.address == trusted_commit.signed_header.header.proposer_address
                            })
                            .expect("proposer must exist in set")
                            .clone(),
                        validators: trusted_validators,
                    },
                    signed_header: untrusted_commit.signed_header,
                };

                print_json(&header);
            }
            SubCmd::FetchProof { path, data, height } => {
                let proof = client
                    .abci_query(path, data, Some(height), true)
                    .await?
                    .decode_merkle_proof()?;

                print_json(&proof);
            }
        }

        Ok(())
    }
}
