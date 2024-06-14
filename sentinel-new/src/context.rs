use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{sync::Mutex, time::interval};
use unionlabs::ethereum::config::{Mainnet, Minimal, PresetBaseKind};

use crate::{
    chains::{Chain, Ethereum, IbcTransfer as _, Osmosis, Union},
    config::{Config, IbcInteraction, KEY_ETHEREUM, KEY_OSMOSIS, KEY_UNION},
};
pub struct Context {
    pub chains: HashMap<String, Chain>,
    pub interactions: Vec<IbcInteraction>,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self, ()> {
        let mut chains = HashMap::new();

        if config.ethereum.enable {
            let eth = match config.ethereum.preset {
                PresetBaseKind::Minimal => {
                    Chain::EthereumMinimal(Ethereum::new(config.ethereum).await)
                }
                PresetBaseKind::Mainnet => {
                    Chain::EthereumMainnet(Ethereum::new(config.ethereum).await)
                }
            };
            chains.insert(KEY_ETHEREUM.to_string(), eth);
        }

        if config.osmosis.enable {
            chains.insert(
                KEY_OSMOSIS.to_string(),
                Chain::Osmosis(Osmosis::new(config.osmosis).await),
            );
        }

        if config.union.enable {
            chains.insert(
                KEY_UNION.to_string(),
                Chain::Union(Union::new(config.union).await),
            );
        }

        Ok(Self {
            chains,
            interactions: config.interactions,
        })
    }

    pub async fn do_transactions(self) {
        for interaction in self.interactions {
            let source_chain = self.chains.get(&interaction.source.chain).cloned().unwrap();
            let destination_chain = self.chains.get(&interaction.destination.chain).unwrap();

            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(interaction.send_packet_interval));
                loop {
                    interval.tick().await;
                    match source_chain {
                        Chain::EthereumMinimal(_) => todo!(),
                        Chain::EthereumMainnet(_) => todo!(),
                        Chain::Osmosis(_) => todo!(),
                        Chain::Union(_) => todo!(),
                    }
                }
            });
        }
    }
}
