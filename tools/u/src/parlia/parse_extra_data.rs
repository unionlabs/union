use anyhow::Result;
use clap::Args;
use unionlabs::primitives::Bytes;

use crate::print_json;

#[derive(Debug, Args)]
pub struct Cmd {
    // TODO: Add block number as an option here
    extra_data: Bytes,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        match parlia_verifier::parse_header_extra_data(&self.extra_data) {
            Ok(ok) => {
                print_json(&ok);
            }
            Err(_) => {
                let res =
                    parlia_verifier::parse_epoch_rotation_header_extra_data(&self.extra_data)?;

                print_json(&res);
            }
        }

        Ok(())
    }
}
