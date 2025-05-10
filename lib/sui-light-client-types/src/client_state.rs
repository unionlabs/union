use crate::committee::Committee;

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "version", content = "data", rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: String,
    pub latest_checkpoint: u64,
    pub frozen_height: u64,
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub initial_committee: Option<Committee>,
}

#[test]
fn encode() {
    let client_state = <ClientState as unionlabs::encoding::DecodeAs>::decode_as::<
        unionlabs::encoding::Bincode,
    >(&hex_literal::hex!(
        "000000000800000000000000346337386164616371ae8e0b00000000000000000000000000"
    ))
    .unwrap();

    println!("client state: {:?}", client_state);

    let consensus_state =
        <crate::consensus_state::ConsensusState as unionlabs::encoding::DecodeAs>::decode_as::<
            unionlabs::encoding::EthAbi,
        >(&hex_literal::hex!(
            "00000000000000000000000000000000000000000000000000000000000003e8"
        ))
        .unwrap();

    println!("consensus state: {:?}", consensus_state);
}
