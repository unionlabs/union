use anyhow::Result;
use clap::{builder::ArgPredicate, Subcommand};
use ibc_union_spec::{path::commit_packets, ChannelId, MustBeZero, Packet, Timestamp};
use unionlabs::{
    encoding::{DecodeAs, Json},
    primitives::Bytes,
};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    BatchHash {
        #[arg(required = true, num_args = 1.., value_parser(|s: &str| serde_json::from_str::<Packet>(s)))]
        packets: Vec<Packet>,
    },
    #[command(visible_alias = "hash")]
    // defaults on the individual field args are arbitrary, they need to have *a* value for if --json is used (and making them optional messes with the cli by making them not required)
    PacketHash {
        #[arg(
            long,
            short = 's',
            visible_alias = "src",
            default_value_if("json", ArgPredicate::IsPresent, "1")
        )]
        source_channel_id: ChannelId,
        #[arg(
            long,
            short = 'd',
            visible_alias = "dst",
            default_value_if("json", ArgPredicate::IsPresent, "1")
        )]
        destination_channel_id: ChannelId,
        #[arg(
            long,
            short = 'D',
            default_value_if("json", ArgPredicate::IsPresent, "0x")
        )]
        data: Bytes,
        #[arg(
            long,
            short = 't',
            default_value_if("json", ArgPredicate::IsPresent, "0")
        )]
        timeout_timestamp: Timestamp,
        /// Provide the full packet as json instead of each field individually.
        #[arg(long, short = 'j', exclusive = true)]
        json: Option<String>,
    },
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::BatchHash { packets } => println!("{}", commit_packets(&packets)),
            Cmd::PacketHash {
                source_channel_id,
                destination_channel_id,
                data,
                timeout_timestamp,
                json,
            } => {
                let hash = match json {
                    Some(json) => Packet::decode_as::<Json>(json.as_bytes())?.hash(),
                    None => Packet {
                        source_channel_id,
                        destination_channel_id,
                        data,
                        timeout_height: MustBeZero,
                        timeout_timestamp,
                    }
                    .hash(),
                };

                println!("{hash}");
            }
        }

        Ok(())
    }
}
