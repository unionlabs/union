use std::process::Command;

use unionlabs::{id::PortId, primitives::Bytes};
use voyager_sdk::{anyhow, primitives::ChainId, serde_json::json};

pub fn create_client(
    on: ChainId,
    tracking: ChainId,
    ibc_interface: String,
    client_type: String,
) -> anyhow::Result<()> {
    Command::new("/home/kaancaglan/dev/union/target/debug/voyager")
        .args([
            "--config-file-path",
            "voyager/config.jsonc",
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

pub fn init_fetch(on: ChainId) -> anyhow::Result<()> {
    Command::new("/home/kaancaglan/dev/union/target/debug/voyager")
        .args([
            "--config-file-path",
            "voyager/config.jsonc",
            "init-fetch",
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
    Command::new("/home/kaancaglan/dev/union/target/debug/voyager")
        .args([
            "--config-file-path",
            "voyager/config.jsonc",
            "q", "e",
            format!("{{\"@type\":\"call\",\"@value\":{{\"@type\":\"submit_tx\",\"@value\":{{\"chain_id\":\"{on}\",\"datagrams\":[{{\"ibc_spec_id\":\"ibc-union\",\"datagram\":{{\"@type\":\"connection_open_init\",\"@value\":{{\"client_id\":{client_id},\"counterparty_client_id\":{counterparty_client_id}}}}}}}]}}}}}}").as_str()
        ]).status()?;

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
    Command::new("/home/kaancaglan/dev/union/target/debug/voyager")
        .args([
            "q",
            "e",
            &json!({
              "@type": "call",
              "@value": {
                "@type": "submit_tx",
                "@value": {
                  "chain_id": chain_id,
                  "datagrams": [
                    {
                      "ibc_spec_id": "ibc-union",
                      "datagram": {
                        "@type": "channel_open_init",
                        "@value": {
                          "port_id": port_id,
                          "counterparty_port_id": counterparty_port_id,
                          "connection_id": connection_id,
                          "version": version
                        }
                      }
                    }
                  ]
                }
              }
            })
            .to_string(),
        ])
        .status()?;

    Ok(())
}
