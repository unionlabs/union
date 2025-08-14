use std::process::Command;

use ibc_union_spec::datagram::{Datagram, MsgChannelOpenInit, MsgConnectionOpenInit};
use unionlabs::primitives::Bytes;
use voyager_sdk::{
    anyhow,
    message::{
        call::{Call, SubmitTx},
        data::IbcDatagram,
        VoyagerMessage,
    },
    primitives::{ChainId, IbcSpecId},
    serde_json::{self},
    vm::Op,
};

pub fn create_client(
    config_file_path: &str,
    on: ChainId,
    tracking: ChainId,
    ibc_interface: String,
    client_type: String,
) -> anyhow::Result<()> {
    Command::new("voyager")
        .args([
            "--config-file-path",
            config_file_path,
            "msg",
            "create-client",
            "--on",
            on.as_str(),
            "--tracking",
            tracking.as_str(),
            "--ibc-interface",
            ibc_interface.as_str(),
            "--ibc-spec-id",
            "ibc-union",
            "--client-type",
            client_type.as_str(),
            "--enqueue",
        ])
        .status()?;
    Ok(())
}

pub fn init_fetch(config_file_path: &str, on: ChainId) -> anyhow::Result<()> {
    Command::new("voyager")
        .args([
            "--config-file-path",
            config_file_path,
            "index",
            on.as_str(),
            "--enqueue",
        ])
        .status()?;
    Ok(())
}

pub fn connection_open(
    on: ChainId,
    client_id: u32,
    counterparty_client_id: u32,
) -> anyhow::Result<()> {
    Command::new("voyager")
        .args([
            "q",
            "e",
            &serde_json::to_string(&Op::<VoyagerMessage>::Call(Call::SubmitTx(SubmitTx {
                chain_id: on,
                datagrams: vec![IbcDatagram {
                    ibc_spec_id: IbcSpecId::new_static(IbcSpecId::UNION),
                    datagram: serde_json::to_value(Datagram::ConnectionOpenInit(
                        MsgConnectionOpenInit {
                            client_id: client_id.try_into().unwrap(),
                            counterparty_client_id: counterparty_client_id.try_into().unwrap(),
                        },
                    ))
                    .unwrap(),
                }],
            })))
            .unwrap(),
        ])
        .status()?;

    Ok(())
}

/// target/debug/voyager q e ''
pub fn channel_open(
    chain_id: ChainId,
    port_id: Bytes,
    counterparty_port_id: Bytes,
    connection_id: u32,
    version: String,
) -> anyhow::Result<()> {
    println!(
        "Opening channel on {}: port_id={}, counterparty_port_id={}, connection_id={}, version={}",
        chain_id, port_id, counterparty_port_id, connection_id, version
    );
    Command::new("voyager")
        .args([
            "q",
            "e",
            &serde_json::to_string(&Op::<VoyagerMessage>::Call(Call::SubmitTx(SubmitTx {
                chain_id,
                datagrams: vec![IbcDatagram {
                    ibc_spec_id: IbcSpecId::new_static(IbcSpecId::UNION),
                    datagram: serde_json::to_value(Datagram::ChannelOpenInit(MsgChannelOpenInit {
                        port_id,
                        counterparty_port_id,
                        connection_id: connection_id.try_into().unwrap(),
                        version,
                    }))
                    .unwrap(),
                }],
            })))
            .unwrap(),
        ])
        .status()?;

    Ok(())
}
