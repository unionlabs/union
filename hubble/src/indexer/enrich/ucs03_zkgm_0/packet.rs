use alloy_sol_types::{sol, SolType};
use anyhow::{Context, Result};
use serde::{
    ser::{Error as SerdeError, SerializeStruct},
    Serialize, Serializer,
};
use serde_json::Value;

// source: github:unionlabs/union/evm/contracts/apps/ucs/03-zkgm/Zkgm.sol
const OP_FORWARD: u8 = 0x00;
const OP_MULTIPLEX: u8 = 0x01;
const OP_BATCH: u8 = 0x02;
const OP_FUNGIBLE_ASSET_TRANSFER: u8 = 0x03;

sol! {
    #[derive(Serialize)]
    struct ZkgmPacket {
        bytes32 salt;
        uint256 path;
        Instruction instruction;
    }

    #[derive(Debug)]
    struct Instruction {
        uint8 version;
        uint8 opcode;
        bytes operand;
    }

    #[derive(Serialize, Debug)]
    struct ForwardV0 {
        uint32 channelId;
        uint64 timeoutHeight;
        uint64 timeoutTimestamp;
        Instruction instruction;
    }

    #[derive(Serialize, Debug)]
    struct MultiplexV0 {
        bytes sender;
        bool eureka;
        bytes contractAddress;
        bytes contractCalldata;
    }

    #[derive(Serialize, Debug)]
    struct BatchV0 {
        Instruction[] instructions;
    }

    #[derive(Serialize, Debug)]
    struct FungibleAssetOrderV0 {
        bytes sender;
        bytes receiver;
        bytes baseToken;
        uint256 baseAmount;
        string baseTokenSymbol;
        string baseTokenName;
        uint256 baseTokenPath;
        bytes quoteToken;
        uint256 quoteAmount;
    }

    #[derive(Serialize, Debug)]
    struct FungibleAssetOrderV1 {
        bytes sender;
        bytes receiver;
        bytes baseToken;
        uint256 baseAmount;
        string baseTokenSymbol;
        string baseTokenName;
        uint8 baseTokenDecimals;
        uint256 baseTokenPath;
        bytes quoteToken;
        uint256 quoteAmount;
    }

    #[derive(Debug)]
    struct FungibleAssetOrderV2 {
        bytes sender;
        bytes receiver;
        bytes base_token;
        uint256 base_amount;
        uint8 metadata_type;
        bytes metadata;
        bytes quote_token;
        uint256 quote_amount;
    }

    #[derive(Serialize, Debug)]
    struct FungibleAssetMetadata {
        bytes implementation;
        bytes initializer;
    }
}

impl Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Create a struct with version, opcode, and operand
        let mut state = serializer.serialize_struct("Instruction", 3)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("opcode", &self.opcode)?;

        // Custom serialization for operand based on version and opcode
        let operand = &self.decode_operand().map_err(|err| {
            S::Error::custom(format!("error decoding operand (in packet): {err}"))
        })?;
        state.serialize_field("operand", &operand)?;

        state.end()
    }
}

impl Serialize for FungibleAssetOrderV2 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Create a struct with the standard fields plus _metadata
        let mut state = serializer.serialize_struct("FungibleAssetOrderV2", 9)?;

        // Serialize all standard fields
        state.serialize_field("sender", &self.sender)?;
        state.serialize_field("receiver", &self.receiver)?;
        state.serialize_field("base_token", &self.base_token)?;
        state.serialize_field("base_amount", &self.base_amount)?;
        state.serialize_field("metadata_type", &self.metadata_type)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.serialize_field("quote_token", &self.quote_token)?;
        state.serialize_field("quote_amount", &self.quote_amount)?;

        // Add _metadata field based on metadata_type
        let metadata_value = match self.metadata_type {
            0 => serde_json::json!({
                "_type": "Image",
                "image": self.metadata
            }),
            1 => {
                // Decode metadata into FungibleAssetMetadata
                match <FungibleAssetMetadata>::abi_decode_sequence(&self.metadata) {
                    Ok(decoded) => serde_json::json!({
                        "_type": "Preimage",
                        "implementation": decoded.implementation,
                        "initializer": decoded.initializer
                    }),
                    Err(_) => serde_json::json!({
                        "_type": "Preimage",
                        "error": "failed to decode metadata"
                    }),
                }
            }
            2 => serde_json::json!({
                "_type": "ImageUnwrap",
                "image": self.metadata
            }),
            _ => serde_json::json!({
                "_type": "Unsupported",
                "data": self.metadata
            }),
        };

        state.serialize_field("_metadata", &metadata_value)?;
        state.end()
    }
}

impl Instruction {
    pub fn decode_operand(&self) -> Result<Operand> {
        Ok(match (self.version, self.opcode) {
            (0, OP_FORWARD) => Operand::Forward(Forward::V0(
                <ForwardV0>::abi_decode_sequence(&self.operand).context("decoding Forward")?,
            )),
            (0, OP_MULTIPLEX) => Operand::Multiplex(Multiplex::V0(
                <MultiplexV0>::abi_decode_sequence(&self.operand).context("decoding Multiplex")?,
            )),
            (0, OP_BATCH) => Operand::Batch(Batch::V0(
                <BatchV0>::abi_decode_sequence(&self.operand).context("decoding Batch")?,
            )),
            (0, OP_FUNGIBLE_ASSET_TRANSFER) => Operand::FungibleAssetOrder(FungibleAssetOrder::V0(
                <FungibleAssetOrderV0>::abi_decode_sequence(&self.operand)
                    .context("decoding FungibleAssetOrderV0")?,
            )),
            (1, OP_FUNGIBLE_ASSET_TRANSFER) => Operand::FungibleAssetOrder(FungibleAssetOrder::V1(
                <FungibleAssetOrderV1>::abi_decode_sequence(&self.operand)
                    .context("decoding FungibleAssetOrderV1")?,
            )),
            (2, OP_FUNGIBLE_ASSET_TRANSFER) => Operand::FungibleAssetOrder(FungibleAssetOrder::V2(
                <FungibleAssetOrderV2>::abi_decode_sequence(&self.operand)
                    .context("decoding FungibleAssetOrderV2")?,
            )),
            _ => Operand::Unsupported {
                data: self.operand.clone(),
            },
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Forward {
    V0(ForwardV0),
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Multiplex {
    V0(MultiplexV0),
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Batch {
    V0(BatchV0),
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum FungibleAssetOrder {
    V0(FungibleAssetOrderV0),
    V1(FungibleAssetOrderV1),
    V2(FungibleAssetOrderV2),
}

#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
pub enum Operand {
    Forward(Forward),
    Multiplex(Multiplex),
    Batch(Batch),
    FungibleAssetOrder(FungibleAssetOrder),
    Unsupported {
        data: alloy_sol_types::private::Bytes,
    },
}

pub fn decode(input: &[u8]) -> Result<Value> {
    let zkgm_packet = <ZkgmPacket>::abi_decode_sequence(input).context("decoding zkgm packet")?;

    let value = serde_json::to_value(&zkgm_packet).context("formatting json")?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v0_packet() {
        let json = decode(&hex::decode("0b00dd4772d3b8ebf5add472a720f986c0846c9b9c1c0ed98f1a011df8486bfc0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002c00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e6831e169d77a861a0e71326afa6d80bcc8bc6aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e6831e169d77a861a0e71326afa6d80bcc8bc6aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000014779877a7b0d9e8603169ddbd7836e478b462478900000000000000000000000000000000000000000000000000000000000000000000000000000000000000044c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f436861696e4c696e6b20546f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d1b482d1b947a96e96c9b76d15de34f7f70a20a1000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "FungibleAssetOrder",
                  "baseAmount": "0x0",
                  "baseToken": "0x779877a7b0d9e8603169ddbd7836e478b4624789",
                  "baseTokenName": "ChainLink Token",
                  "baseTokenPath": "0x0",
                  "baseTokenSymbol": "LINK",
                  "quoteAmount": "0x0",
                  "quoteToken": "0xd1b482d1b947a96e96c9b76d15de34f7f70a20a1",
                  "receiver": "0xe6831e169d77a861a0e71326afa6d80bcc8bc6aa",
                  "sender": "0xe6831e169d77a861a0e71326afa6d80bcc8bc6aa"
                },
                "version": 0
              },
              "path": "0x0",
              "salt": "0x0b00dd4772d3b8ebf5add472a720f986c0846c9b9c1c0ed98f1a011df8486bfc"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v1_packet() {
        let json = decode(&hex::decode("dddde7c9642e7ecbb7bbe659eff187e8ee6691fd7c840b09a89ec6b126caaaaa000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014acbd24742b87c34ded607fb87b22401b2ede167e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756e696f6e31677968347464377639366d7563723465616b7364326d7367306a76306d636e396135796a38357678356c376874793374753970737178736a79320000000000000000000000000000000000000000000000000000000000000004414e414d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000963616e696d616e616d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014a56815e60b4e13b937c91ddee677fae1d3955466000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "FungibleAssetOrder",
                  "baseAmount": "0xa",
                  "baseToken": "0x756e696f6e31677968347464377639366d7563723465616b7364326d7367306a76306d636e396135796a38357678356c376874793374753970737178736a7932",
                  "baseTokenDecimals": 8,
                  "baseTokenName": "canimanam",
                  "baseTokenPath": "0x0",
                  "baseTokenSymbol": "ANAM",
                  "quoteAmount": "0xa",
                  "quoteToken": "0xa56815e60b4e13b937c91ddee677fae1d3955466",
                  "receiver": "0xacbd24742b87c34ded607fb87b22401b2ede167e",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732"
                },
                "version": 1
              },
              "path": "0x0",
              "salt": "0xdddde7c9642e7ecbb7bbe659eff187e8ee6691fd7c840b09a89ec6b126caaaaa"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v2_packet_metadata_type_image() {
        let json = decode(&hex::decode("b4536add4924363adf36c5525508616d702ea6c1e60b6544cd1b542f761a02ab0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db6192800000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_metadata": {
                    "_type": "Image",
                    "image": "0x457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280"
                  },
                  "_type": "FungibleAssetOrder",
                  "base_amount": "0x64",
                  "base_token": "0x6d756e6f",
                  "metadata": "0x457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280",
                  "metadata_type": 0,
                  "quote_amount": "0x64",
                  "quote_token": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732"
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb4536add4924363adf36c5525508616d702ea6c1e60b6544cd1b542f761a02ab"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v2_packet_metadata_type_preimage() {
        let json = decode(&hex::decode("b515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae780000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000004800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000004400000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000149c968b805a625303ad43fce99ae72306256fe5f9000000000000000000000000000000000000000000000000000000000000000000000000000000000000018499f0385300000000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "FungibleAssetOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "base_token": "0x6d756e6f",
                  "base_amount": "0x64",
                  "metadata_type": 1,
                  "metadata": "0x0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000149c968b805a625303ad43fce99ae72306256fe5f9000000000000000000000000000000000000000000000000000000000000000000000000000000000000018499f0385300000000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a00000000000000000000000000000000000000000000000000000000000000000000000000000000",
                  "quote_token": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quote_amount": "0x64",
                  "_metadata": {
                    "_type": "Preimage",
                    "implementation": "0x9c968b805a625303ad43fce99ae72306256fe5f9",
                    "initializer": "0x99f0385300000000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a000000000000000000000000"
                  }
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v2_packet_metadata_type_image_unwrap() {
        // Create a V2 packet with metadata_type = 2 (ImageUnwrap) using real hex encoding
        // replace with hex once packet is on chain
        use alloy_sol_types::SolType;

        let v2_order = FungibleAssetOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            base_token: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            base_amount: alloy_sol_types::private::U256::from(100u64),
            metadata_type: 2,
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("cafebabecafebabecafebabecafebabecafebabe").unwrap()),
            quote_token: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quote_amount: alloy_sol_types::private::U256::from(100u64),
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_FUNGIBLE_ASSET_TRANSFER,
            operand: <FungibleAssetOrderV2>::abi_encode_sequence(&v2_order).into(),
        };

        let salt_bytes =
            hex::decode("b515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78")
                .unwrap();
        let mut salt_array = [0u8; 32];
        salt_array.copy_from_slice(&salt_bytes);

        let zkgm_packet = ZkgmPacket {
            salt: alloy_sol_types::private::FixedBytes::from(salt_array),
            path: alloy_sol_types::private::U256::from(0u64),
            instruction,
        };

        let encoded = <ZkgmPacket>::abi_encode_sequence(&zkgm_packet);
        let hex_string = hex::encode(encoded);

        // Use the generated hex for the test
        let json = decode(&hex::decode(hex_string).unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "FungibleAssetOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "base_token": "0x6d756e6f",
                  "base_amount": "0x64",
                  "metadata_type": 2,
                  "metadata": "0xcafebabecafebabecafebabecafebabecafebabe",
                  "quote_token": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quote_amount": "0x64",
                  "_metadata": {
                    "_type": "ImageUnwrap",
                    "image": "0xcafebabecafebabecafebabecafebabecafebabe"
                  }
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v2_packet_unsupported_metadata_type() {
        // Create a V2 packet with metadata_type = 99 (unsupported) using real hex encoding
        use alloy_sol_types::SolType;

        let v2_order = FungibleAssetOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            base_token: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            base_amount: alloy_sol_types::private::U256::from(100u64),
            metadata_type: 99,
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap()),
            quote_token: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quote_amount: alloy_sol_types::private::U256::from(100u64),
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_FUNGIBLE_ASSET_TRANSFER,
            operand: <FungibleAssetOrderV2>::abi_encode_sequence(&v2_order).into(),
        };

        let salt_bytes =
            hex::decode("b515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78")
                .unwrap();
        let mut salt_array = [0u8; 32];
        salt_array.copy_from_slice(&salt_bytes);

        let zkgm_packet = ZkgmPacket {
            salt: alloy_sol_types::private::FixedBytes::from(salt_array),
            path: alloy_sol_types::private::U256::from(0u64),
            instruction,
        };

        let encoded = <ZkgmPacket>::abi_encode_sequence(&zkgm_packet);
        let hex_string = hex::encode(encoded);

        // Use the generated hex for the test
        let json = decode(&hex::decode(hex_string).unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "FungibleAssetOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "base_token": "0x6d756e6f",
                  "base_amount": "0x64",
                  "metadata_type": 99,
                  "metadata": "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
                  "quote_token": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quote_amount": "0x64",
                  "_metadata": {
                    "_type": "Unsupported",
                    "data": "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
                  }
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_fungible_asset_transfer_v2_packet_unparseable_metadata() {
        // Create a V2 packet with metadata_type = 1 but invalid metadata that cannot be parsed
        use alloy_sol_types::SolType;

        let v2_order = FungibleAssetOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            base_token: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            base_amount: alloy_sol_types::private::U256::from(100u64),
            metadata_type: 1,
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("deadbeef").unwrap()), // Invalid/too short for FungibleAssetMetadata
            quote_token: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quote_amount: alloy_sol_types::private::U256::from(100u64),
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_FUNGIBLE_ASSET_TRANSFER,
            operand: <FungibleAssetOrderV2>::abi_encode_sequence(&v2_order).into(),
        };

        let salt_bytes =
            hex::decode("b515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78")
                .unwrap();
        let mut salt_array = [0u8; 32];
        salt_array.copy_from_slice(&salt_bytes);

        let zkgm_packet = ZkgmPacket {
            salt: alloy_sol_types::private::FixedBytes::from(salt_array),
            path: alloy_sol_types::private::U256::from(0u64),
            instruction,
        };

        let encoded = <ZkgmPacket>::abi_encode_sequence(&zkgm_packet);
        let hex_string = hex::encode(encoded);

        // Use the generated hex for the test
        let json = decode(&hex::decode(hex_string).unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "FungibleAssetOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "base_token": "0x6d756e6f",
                  "base_amount": "0x64",
                  "metadata_type": 1,
                  "metadata": "0xdeadbeef",
                  "quote_token": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quote_amount": "0x64",
                  "_metadata": {
                    "_type": "Preimage",
                    "error": "failed to decode metadata"
                  }
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_xyz() {
        let result = decode(&hex::decode("00").unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_batch() {
        let json = decode(&hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014dc7af843e4eb079cd77ace6774bd71d6b8122f07000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a666163746f72792f756e696f6e31327164766d7732326e37326d656d3079736666336e6c796a32633736637579347836306c75612f636c6f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000148b4bfb23f4d75feef28b4099c0114e5840d14a4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014271126f4f9b36ce16d9e2ef75691485ddce11db60000000000000000000000000000000000000000000000000000000000000000000000000000000000000004cafebabe00000000000000000000000000000000000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 2,
                "operand": {
                  "_type": "Batch",
                  "instructions": [
                    {
                      "opcode": 3,
                      "operand": {
                        "_type": "FungibleAssetOrder",
                        "baseAmount": "0x1",
                        "baseToken": "0xdc7af843e4eb079cd77ace6774bd71d6b8122f07",
                        "baseTokenName": "",
                        "baseTokenPath": "0x0",
                        "baseTokenSymbol": "factory/union12qdvmw22n72mem0ysff3nlyj2c76cuy4x60lua/clown",
                        "quoteAmount": "0x1",
                        "quoteToken": "0x8b4bfb23f4d75feef28b4099c0114e5840d14a47",
                        "receiver": "0x153919669edc8a5d0c8d1e4507c9ce60435a1177",
                        "sender": "0x153919669edc8a5d0c8d1e4507c9ce60435a1177"
                      },
                      "version": 0
                    },
                    {
                      "opcode": 1,
                      "operand": {
                        "_type": "Multiplex",
                        "contractAddress": "0x271126f4f9b36ce16d9e2ef75691485ddce11db6",
                        "contractCalldata": "0xcafebabe",
                        "eureka": true,
                        "sender": "0x153919669edc8a5d0c8d1e4507c9ce60435a1177"
                      },
                      "version": 0
                    }
                  ]
                },
                "version": 0
              },
              "path": "0x0",
              "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
            })
        );
    }

    #[test]
    fn test_complex() {
        let json = decode(&hex::decode("123456789012345678901234567890123456789012345678901234567890123400000000000000000000000000000000000000000000000000000000075bcd1500000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000b0000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003c0000000000000000000000000000000000000000000000000000000000000058000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001474d5b8eacfeb0dadaaf66403f40e304b3ef968b300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD000000000000000000000000000000000000000000000000000000000000000000000000000000000000001474d5b8eacfeb0dadaaf66403f40e304b3ef968b30000000000000000000000000000000000000000000000000000000000000000000000000000000000000018736f6d6520736d61727420636f6e74726163742064617461000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000004c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009896800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001474d5b8eacfeb0dadaaf66403f40e304b3ef968b3000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "path": "0x75bcd15",
              "salt": "0x1234567890123456789012345678901234567890123456789012345678901234",
              "instruction": {
                "opcode": 2,
                "operand": {
                  "_type": "Batch",
                  "instructions": [
                    {
                      "opcode": 3,
                      "operand": {
                        "_type": "FungibleAssetOrder",
                        "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                        "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                        "baseToken": "0x6d756e6f",
                        "baseAmount": "0x4",
                        "quoteToken": "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
                        "quoteAmount": "0x4",
                        "baseTokenName": "muno",
                        "baseTokenPath": "0x0",
                        "baseTokenSymbol": "muno",
                        "baseTokenDecimals": 18
                      },
                      "version": 1
                    },
                    {
                      "opcode": 1,
                      "operand": {
                        "_type": "Multiplex",
                        "eureka": true,
                        "sender": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                        "contractAddress": "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
                        "contractCalldata": "0x736f6d6520736d61727420636f6e74726163742064617461"
                      },
                      "version": 0
                    },
                    {
                      "opcode": 0,
                      "operand": {
                        "_type": "Forward",
                        "channelId": 0,
                        "instruction": {
                          "opcode": 2,
                          "operand": {
                            "_type": "Batch",
                            "instructions": [
                              {
                                "opcode": 3,
                                "operand": {
                                  "_type": "FungibleAssetOrder",
                                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                                  "baseToken": "0x6d756e6f",
                                  "baseAmount": "0x4",
                                  "quoteToken": "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
                                  "quoteAmount": "0x4",
                                  "baseTokenName": "muno",
                                  "baseTokenPath": "0x0",
                                  "baseTokenSymbol": "muno",
                                  "baseTokenDecimals": 18
                                },
                                "version": 1
                              }
                            ]
                          },
                          "version": 0
                        },
                        "timeoutHeight": 10000000,
                        "timeoutTimestamp": 0
                      },
                      "version": 0
                    }
                  ]
                },
                "version": 0
              }
            })
        );
    }
}
