use alloy_sol_types::{sol, SolType};
use anyhow::{Context, Result};
use serde::{
    ser::{Error as SerdeError, SerializeStruct},
    Serialize, Serializer,
};
use serde_json::{json, Value};

// source: github:unionlabs/union/evm/contracts/apps/ucs/03-zkgm/Zkgm.sol
const OP_FORWARD: u8 = 0x00;
const OP_CALL: u8 = 0x01;
const OP_BATCH: u8 = 0x02;
const OP_TOKEN_ORDER: u8 = 0x03;

const TOKEN_ORDER_KIND_INITIALIZE: u8 = 0x00;
const TOKEN_ORDER_KIND_ESCROW: u8 = 0x01;
const TOKEN_ORDER_KIND_UNESCROW: u8 = 0x02;
const TOKEN_ORDER_KIND_SOLVE: u8 = 0x03;

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
    struct CallV0 {
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
    struct TokenOrderV0 {
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
    struct TokenOrderV1 {
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
    struct TokenOrderV2 {
        bytes sender;
        bytes receiver;
        bytes baseToken;
        uint256 baseAmount;
        bytes quoteToken;
        uint256 quoteAmount;
        uint8 kind;
        bytes metadata;
    }

    #[derive(Serialize, Debug)]
    struct TokenMetadata {
        bytes implementation;
        bytes initializer;
    }

    #[derive(Serialize, Debug)]
    struct SolveMetadata {
        bytes solverAddress;
        bytes metadata;
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

impl Serialize for TokenOrderV2 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Create a struct with the standard fields plus _metadata
        let mut state = serializer.serialize_struct("FungibleAssetOrderV2", 9)?;

        // Serialize all standard fields
        state.serialize_field("sender", &self.sender)?;
        state.serialize_field("receiver", &self.receiver)?;
        state.serialize_field("baseToken", &self.baseToken)?;
        state.serialize_field("baseAmount", &self.baseAmount)?;
        state.serialize_field("quoteToken", &self.quoteToken)?;
        state.serialize_field("quoteAmount", &self.quoteAmount)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("metadata", &self.metadata)?;

        // Add _metadata field based on metadata_type
        let metadata_value = match self.kind {
            TOKEN_ORDER_KIND_INITIALIZE => {
                // Decode metadata into TokenMetadata
                match <TokenMetadata>::abi_decode_sequence(&self.metadata) {
                    Ok(decoded) => json!({
                        "_type": "Initialize",
                        "implementation": decoded.implementation,
                        "initializer": decoded.initializer
                    }),
                    Err(_) => json!({
                        "_type": "Initialize",
                        "error": "failed to decode token metadata"
                    }),
                }
            }
            TOKEN_ORDER_KIND_ESCROW => json!({
                "_type": "Escrow",
                "data": self.metadata
            }),
            TOKEN_ORDER_KIND_UNESCROW => json!({
                "_type": "Unescrow",
                "data": self.metadata
            }),
            TOKEN_ORDER_KIND_SOLVE => {
                // Decode metadata into SolveMetadata
                match <SolveMetadata>::abi_decode_sequence(&self.metadata) {
                    Ok(decoded) => json!({
                        "_type": "Solve",
                        "solverAddress": decoded.solverAddress,
                        "metadata": decoded.metadata
                    }),
                    Err(_) => json!({
                        "_type": "Solve",
                        "error": "failed to decode solve metadata"
                    }),
                }
            }
            _ => json!({
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
            (0, OP_CALL) => Operand::Call(Call::V0(
                <CallV0>::abi_decode_sequence(&self.operand).context("decoding Call")?,
            )),
            (0, OP_BATCH) => Operand::Batch(Batch::V0(
                <BatchV0>::abi_decode_sequence(&self.operand).context("decoding Batch")?,
            )),
            (0, OP_TOKEN_ORDER) => Operand::TokenOrder(TokenOrder::V0(
                <TokenOrderV0>::abi_decode_sequence(&self.operand)
                    .context("decoding TokenOrderV0")?,
            )),
            (1, OP_TOKEN_ORDER) => Operand::TokenOrder(TokenOrder::V1(
                <TokenOrderV1>::abi_decode_sequence(&self.operand)
                    .context("decoding TokenOrderV1")?,
            )),
            (2, OP_TOKEN_ORDER) => Operand::TokenOrder(TokenOrder::V2(
                <TokenOrderV2>::abi_decode_sequence(&self.operand)
                    .context("decoding TokenOrderV2")?,
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
pub enum Call {
    V0(CallV0),
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Batch {
    V0(BatchV0),
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum TokenOrder {
    V0(TokenOrderV0),
    V1(TokenOrderV1),
    V2(TokenOrderV2),
}

#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
pub enum Operand {
    Forward(Forward),
    Call(Call),
    Batch(Batch),
    TokenOrder(TokenOrder),
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v0_packet() {
        let json = decode(&hex::decode("0b00dd4772d3b8ebf5add472a720f986c0846c9b9c1c0ed98f1a011df8486bfc0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002c00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e6831e169d77a861a0e71326afa6d80bcc8bc6aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e6831e169d77a861a0e71326afa6d80bcc8bc6aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000014779877a7b0d9e8603169ddbd7836e478b462478900000000000000000000000000000000000000000000000000000000000000000000000000000000000000044c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f436861696e4c696e6b20546f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d1b482d1b947a96e96c9b76d15de34f7f70a20a1000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "TokenOrder",
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v1_packet() {
        let json = decode(&hex::decode("dddde7c9642e7ecbb7bbe659eff187e8ee6691fd7c840b09a89ec6b126caaaaa000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014acbd24742b87c34ded607fb87b22401b2ede167e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756e696f6e31677968347464377639366d7563723465616b7364326d7367306a76306d636e396135796a38357678356c376874793374753970737178736a79320000000000000000000000000000000000000000000000000000000000000004414e414d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000963616e696d616e616d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014a56815e60b4e13b937c91ddee677fae1d3955466000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "TokenOrder",
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_kind_escrow() {
        let json = decode(&hex::decode("b4536add4924363adf36c5525508616d702ea6c1e60b6544cd1b542f761a02ab0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba0000000000000000000000000000000000000000000000000000000000000000000000000000000000000020457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_metadata": {
                    "_type": "Escrow",
                    "data": "0x457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280"
                  },
                  "_type": "TokenOrder",
                  "baseAmount": "0x64",
                  "baseToken": "0x6d756e6f",
                  "quoteAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "kind": 1,
                  "metadata": "0x457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280"
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb4536add4924363adf36c5525508616d702ea6c1e60b6544cd1b542f761a02ab"
            })
        );
    }

    #[test]
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_kind_initialize() {
        let json = decode(&hex::decode("b515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae780000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000004800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba00000000000000000000000000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000149c968b805a625303ad43fce99ae72306256fe5f900000000000000000000000000000000000000000000000000000000000000000000000000000000000001829f0385300000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_type": "TokenOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "baseToken": "0x6d756e6f",
                  "baseAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quoteAmount": "0x64",
                  "kind": 0,
                  "metadata": "0x0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000149c968b805a625303ad43fce99ae72306256fe5f900000000000000000000000000000000000000000000000000000000000000000000000000000000000001829f0385300000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                  "_metadata": {
                    "_type": "Initialize",
                    "implementation": "0x9c968b805a625303ad43fce99ae72306256fe5f9",
                    "initializer": "0x9f0385300000000000000000000040cdff51ae7487e0b4a4d6e5f86eb15fb7c1d9f40000000000000000000000005fbe74a283f7954f10aa04c2edf55578811aeb0300000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000005556e696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001550000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140b885dae80342524f34d46b19744e304ec88c99a000000000000000000000000"
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_kind_unescrow() {
        // Create a V2 packet with metadata_type = 2 (ImageUnwrap) using real hex encoding
        // replace with hex once packet is on chain
        use alloy_sol_types::SolType;

        let v2_order = TokenOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            baseToken: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            baseAmount: alloy_sol_types::private::U256::from(100u64),
            quoteToken: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quoteAmount: alloy_sol_types::private::U256::from(100u64),
            kind: 2, // unescrow
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("cafebabecafebabecafebabecafebabecafebabe").unwrap()),
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_TOKEN_ORDER,
            operand: <TokenOrderV2>::abi_encode_sequence(&v2_order).into(),
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
                  "_type": "TokenOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "baseToken": "0x6d756e6f",
                  "baseAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quoteAmount": "0x64",
                  "kind": 2,
                  "metadata": "0xcafebabecafebabecafebabecafebabecafebabe",
                  "_metadata": {
                    "_type": "Unescrow",
                    "data": "0xcafebabecafebabecafebabecafebabecafebabe"
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_kind_solve() {
        // Create a V2 packet with metadata_type = 3 (Solve) using real hex encoding
        // replace with hex once packet is on chain
        let v2_order = TokenOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            baseToken: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            baseAmount: alloy_sol_types::private::U256::from(100u64),
            quoteToken: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quoteAmount: alloy_sol_types::private::U256::from(100u64),
            kind: 3, // solve
            metadata: SolveMetadata::abi_encode_params(&SolveMetadata { solverAddress: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()), metadata: alloy_sol_types::private::Bytes::from(hex::decode("deadbeef").unwrap())}).into(),
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_TOKEN_ORDER,
            operand: <TokenOrderV2>::abi_encode_sequence(&v2_order).into(),
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
                  "_type": "TokenOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "baseToken": "0x6d756e6f",
                  "baseAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quoteAmount": "0x64",
                  "kind": 3,
                  "metadata": "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba0000000000000000000000000000000000000000000000000000000000000000000000000000000000000004deadbeef00000000000000000000000000000000000000000000000000000000",
                  "_metadata": {
                    "_type": "Solve",
                    "solverAddress": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                    "metadata": "0xdeadbeef"
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_unsupported_kind() {
        // Create a V2 packet with metadata_type = 99 (unsupported) using real hex encoding
        use alloy_sol_types::SolType;

        let v2_order = TokenOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            baseToken: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            baseAmount: alloy_sol_types::private::U256::from(100u64),
            quoteToken: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quoteAmount: alloy_sol_types::private::U256::from(100u64),
            kind: 99, // unsupported
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap()),
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_TOKEN_ORDER,
            operand: <TokenOrderV2>::abi_encode_sequence(&v2_order).into(),
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
                  "_type": "TokenOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "baseToken": "0x6d756e6f",
                  "baseAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quoteAmount": "0x64",
                  "kind": 99,
                  "metadata": "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_initialize_unparsable_metadata() {
        // Create a V2 packet with metadata_type = 0 (initialize) but invalid metadata that cannot be parsed
        use alloy_sol_types::SolType;

        let v2_order = TokenOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            baseToken: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            baseAmount: alloy_sol_types::private::U256::from(100u64),
            quoteToken: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quoteAmount: alloy_sol_types::private::U256::from(100u64),
            kind: 0, // Initialize
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("deadbeef").unwrap()), // Invalid/too short for TokenOrderMetadata
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_TOKEN_ORDER,
            operand: <TokenOrderV2>::abi_encode_sequence(&v2_order).into(),
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
                  "_type": "TokenOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "baseToken": "0x6d756e6f",
                  "baseAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quoteAmount": "0x64",
                  "kind": 0,
                  "metadata": "0xdeadbeef",
                  "_metadata": {
                    "_type": "Initialize",
                    "error": "failed to decode token metadata"
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
    fn test_parse_ucs03_zkgm_0_with_token_order_v2_packet_solve_unparsable_metadata() {
        // Create a V2 packet with metadata_type = 3 (solve) but invalid metadata that cannot be parsed
        use alloy_sol_types::SolType;

        let v2_order = TokenOrderV2 {
            sender: alloy_sol_types::private::Bytes::from(hex::decode("756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732").unwrap()),
            receiver: alloy_sol_types::private::Bytes::from(hex::decode("be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap()),
            baseToken: alloy_sol_types::private::Bytes::from(hex::decode("6d756e6f").unwrap()),
            baseAmount: alloy_sol_types::private::U256::from(100u64),
            quoteToken: alloy_sol_types::private::Bytes::from(hex::decode("ba53d2414765913e7b0b47c3ab3fc1e81006e7ba").unwrap()),
            quoteAmount: alloy_sol_types::private::U256::from(100u64),
            kind: 3, // Solve
            metadata: alloy_sol_types::private::Bytes::from(hex::decode("deadbeef").unwrap()), // Invalid/too short for TokenOrderMetadata
        };

        let instruction = Instruction {
            version: 2,
            opcode: OP_TOKEN_ORDER,
            operand: <TokenOrderV2>::abi_encode_sequence(&v2_order).into(),
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
                  "_type": "TokenOrder",
                  "sender": "0x756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a347974326732",
                  "receiver": "0xbe68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed",
                  "baseToken": "0x6d756e6f",
                  "baseAmount": "0x64",
                  "quoteToken": "0xba53d2414765913e7b0b47c3ab3fc1e81006e7ba",
                  "quoteAmount": "0x64",
                  "kind": 3,
                  "metadata": "0xdeadbeef",
                  "_metadata": {
                    "_type": "Solve",
                    "error": "failed to decode solve metadata"
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
                        "_type": "TokenOrder",
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
                        "_type": "Call",
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
                        "_type": "TokenOrder",
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
                        "_type": "Call",
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
                                  "_type": "TokenOrder",
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

    #[test]
    fn test_bond() {
        let instruction = Instruction {
            version: 0,
            opcode: OP_BATCH,
            operand: hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000003e000000000000000000000000000000000000000000000000000000000000007400000000000000000000000000000000000000000000000000000000000000a2000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca487000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba5eD44733953d79717F6269357C77718C8Ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000002617500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca487000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001837b22636f6e7472616374223a2230783735366536393666366533313635373536353735363537353635373533393736363137323334373936383634373237353739376136623661363337333638333733343738376136353735363733363633366237393739333633303638373333303736363337313665376137313663333236383731333036633738363333323636222c226d7367223a2265794a746157353058335276496a6f694d4867334e545a6c4e6a6b325a6a5a6c4d7a457a4d6a4d794e6d55334f544d7a4e6d51324e5463774d7a4932597a4d334e6d55324f4463304e6a45324e6a63774e7a63324d5463324d7a49334f544d354e6a557a4e545a684e7a49334d7a5a6a4e6a67324e545a694d7a637a4e6a59344e7a4d3259545a6a4969776962576c7558323170626e52665957317664573530496a6f694d534a39222c2266756e6473223a5b7b2264656e6f6d223a226175222c22616d6f756e74223a2231227d5d2c2263616c6c5f616374696f6e223a2263616c6c5f70726f7879227d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca487000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001187b22636f6e7472616374223a2230783735366536393666366533313635373536353735363537353635373533393736363137323334373936383634373237353739376136623661363337333638333733343738376136353735363733363633366237393739333633303638373333303736363337313665376137313663333236383731333036633738363333323636222c226d7367223a2265794a7a634756755a475679496a6f694d4867315a6d4a6c4e7a52684d6a677a5a6a63354e54526d4d54426859544130597a4a6c5a4759314e5455334f4467784d57466c596a417a496977695957317664573530496a6f694d534a39222c2266756e6473223a5b5d2c2263616c6c5f616374696f6e223a22646972656374227d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000ba00000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca487000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a607b22636f6e7472616374223a22307835666265373461323833663739353466313061613034633265646635353537383831316165623033222c226d7367223a2265794a775958526f496a6f694d434973496d4e6f595735755a577866615751694f6a45354c434a7a59577830496a6f694d48686c5954646a4e6d566d4d7a4d325a5467354e3251334e4749305a474e6d4d444d7a4f444d795a44677959324534597a6c684e5755344d57566a4f546b354e5468695a444d344d6a4933597a6c694e7a63775a474535496977696157357a64484a3159335270623234694f694977654441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d5441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444178595441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441784d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d6d4d334e545a6c4e6a6b325a6a5a6c4d7a457a4d6a4d794e6d55334f544d7a4e6d51324e5463774d7a4932597a4d334e6d55324f4463304e6a45324e6a63774e7a63324d5463324d7a49334f544d354e6a557a4e545a684e7a49334d7a5a6a4e6a67324e545a694d7a637a4e6a59344e7a4d3259545a6a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d5451774e6a59794e7a63784e47597a526a4533595463774d5759334d44633059544579517a41794f445133595456454d6b4e684e4467334d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4463314e6d55324f545a6d4e6d557a4d5459314e7a55324e5463314e6a55334e5459314e7a557a4f5463324e6a45334d6a4d304e7a6b324f4459304e7a49334e5463354e324532596a5a684e6a4d334d7a59344d7a637a4e4463344e3245324e5463314e6a637a4e6a597a4e6d49334f5463354d7a597a4d4459344e7a4d7a4d4463324e6a4d334d545a6c4e3245334d545a6a4d7a49324f4463784d7a4132597a63344e6a4d7a4d6a59324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441784e475531513259784d304d344e474d775a6b56684d7a497a4e6b4d784d4446435a44646b4e7a517a5a444d774d7a593252545644526a45774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4745774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774f4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444530596d45315a5751304e44637a4d7a6b314d3251334f5463784e3259324d6a59354d7a5533597a63334e7a4534597a68695954566c5a4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44416966513d3d222c2266756e6473223a5b5d2c2263616c6c5f616374696f6e223a22646972656374227d").unwrap().into(),
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

        let json = decode(&encoded).unwrap();

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
                        "_metadata": {
                          "_type": "Solve",
                          "metadata": "0x",
                          "solverAddress": "0xba5ed44733953d79717f6269357c77718c8ba5ed"
                        },
                        "_type": "TokenOrder",
                        "baseAmount": "0x1",
                        "baseToken": "0xba5ed44733953d79717f6269357c77718c8ba5ed",
                        "kind": 3,
                        "metadata": "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                        "quoteAmount": "0x1",
                        "quoteToken": "0x6175",
                        "receiver": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c",
                        "sender": "0x06627714f3f17a701f7074a12c02847a5d2ca487"
                      },
                      "version": 2
                    },
                    {
                      "opcode": 1,
                      "operand": {
                        "_type": "Call",
                        "contractAddress": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c",
                        "contractCalldata": "0x7b22636f6e7472616374223a2230783735366536393666366533313635373536353735363537353635373533393736363137323334373936383634373237353739376136623661363337333638333733343738376136353735363733363633366237393739333633303638373333303736363337313665376137313663333236383731333036633738363333323636222c226d7367223a2265794a746157353058335276496a6f694d4867334e545a6c4e6a6b325a6a5a6c4d7a457a4d6a4d794e6d55334f544d7a4e6d51324e5463774d7a4932597a4d334e6d55324f4463304e6a45324e6a63774e7a63324d5463324d7a49334f544d354e6a557a4e545a684e7a49334d7a5a6a4e6a67324e545a694d7a637a4e6a59344e7a4d3259545a6a4969776962576c7558323170626e52665957317664573530496a6f694d534a39222c2266756e6473223a5b7b2264656e6f6d223a226175222c22616d6f756e74223a2231227d5d2c2263616c6c5f616374696f6e223a2263616c6c5f70726f7879227d",
                        "eureka": false,
                        "sender": "0x06627714f3f17a701f7074a12c02847a5d2ca487"
                      },
                      "version": 0
                    },
                    {
                      "opcode": 1,
                      "operand": {
                        "_type": "Call",
                        "contractAddress": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c",
                        "contractCalldata": "0x7b22636f6e7472616374223a2230783735366536393666366533313635373536353735363537353635373533393736363137323334373936383634373237353739376136623661363337333638333733343738376136353735363733363633366237393739333633303638373333303736363337313665376137313663333236383731333036633738363333323636222c226d7367223a2265794a7a634756755a475679496a6f694d4867315a6d4a6c4e7a52684d6a677a5a6a63354e54526d4d54426859544130597a4a6c5a4759314e5455334f4467784d57466c596a417a496977695957317664573530496a6f694d534a39222c2266756e6473223a5b5d2c2263616c6c5f616374696f6e223a22646972656374227d",
                        "eureka": false,
                        "sender": "0x06627714f3f17a701f7074a12c02847a5d2ca487"
                      },
                      "version": 0
                    },
                    {
                      "opcode": 1,
                      "operand": {
                        "_type": "Call",
                        "contractAddress": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c",
                        "contractCalldata": "0x7b22636f6e7472616374223a22307835666265373461323833663739353466313061613034633265646635353537383831316165623033222c226d7367223a2265794a775958526f496a6f694d434973496d4e6f595735755a577866615751694f6a45354c434a7a59577830496a6f694d48686c5954646a4e6d566d4d7a4d325a5467354e3251334e4749305a474e6d4d444d7a4f444d795a44677959324534597a6c684e5755344d57566a4f546b354e5468695a444d344d6a4933597a6c694e7a63775a474535496977696157357a64484a3159335270623234694f694977654441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d5441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444178595441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441784d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d6d4d334e545a6c4e6a6b325a6a5a6c4d7a457a4d6a4d794e6d55334f544d7a4e6d51324e5463774d7a4932597a4d334e6d55324f4463304e6a45324e6a63774e7a63324d5463324d7a49334f544d354e6a557a4e545a684e7a49334d7a5a6a4e6a67324e545a694d7a637a4e6a59344e7a4d3259545a6a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d5451774e6a59794e7a63784e47597a526a4533595463774d5759334d44633059544579517a41794f445133595456454d6b4e684e4467334d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4463314e6d55324f545a6d4e6d557a4d5459314e7a55324e5463314e6a55334e5459314e7a557a4f5463324e6a45334d6a4d304e7a6b324f4459304e7a49334e5463354e324532596a5a684e6a4d334d7a59344d7a637a4e4463344e3245324e5463314e6a637a4e6a597a4e6d49334f5463354d7a597a4d4459344e7a4d7a4d4463324e6a4d334d545a6c4e3245334d545a6a4d7a49324f4463784d7a4132597a63344e6a4d7a4d6a59324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441784e475531513259784d304d344e474d775a6b56684d7a497a4e6b4d784d4446435a44646b4e7a517a5a444d774d7a593252545644526a45774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4745774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774f4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444530596d45315a5751304e44637a4d7a6b314d3251334f5463784e3259324d6a59354d7a5533597a63334e7a4534597a68695954566c5a4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44416966513d3d222c2266756e6473223a5b5d2c2263616c6c5f616374696f6e223a22646972656374227d",
                        "eureka": false,
                        "sender": "0x06627714f3f17a701f7074a12c02847a5d2ca487"
                      },
                      "version": 0
                    }
                  ]
                },
                "version": 0
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }

    #[test]
    fn test_bond_last_call_instruction() {
        let instruction = <Instruction>::abi_decode_sequence(&hex::decode("00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000240000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca4870000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756e696f6e31657565756575657539766172347968647275797a6b6a6373683734787a65756736636b797936306873307663716e7a716c326871306c786332660000000000000000000000000000000000000000000000000000000000000014e5Cf13C84c0fEa3236C101Bd7d743d30366E5CF100000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()).unwrap();

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

        let json = decode(&encoded).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "opcode": 3,
                "operand": {
                  "_metadata": {
                    "_type": "Solve",
                    "metadata": "0x",
                    "solverAddress": "0xba5ed44733953d79717f6269357c77718c8ba5ed"
                  },
                  "_type": "TokenOrder",
                  "baseAmount": "0x1",
                  "baseToken": "0x756e696f6e31657565756575657539766172347968647275797a6b6a6373683734787a65756736636b797936306873307663716e7a716c326871306c78633266",
                  "kind": 3,
                  "metadata": "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                  "quoteAmount": "0x1",
                  "quoteToken": "0xe5cf13c84c0fea3236c101bd7d743d30366e5cf1",
                  "receiver": "0x06627714f3f17a701f7074a12c02847a5d2ca487",
                  "sender": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c"
                },
                "version": 2
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }

    #[test]
    fn test_unbond() {
        let instruction = Instruction {
            version: 0,
            opcode: OP_BATCH,
            operand: hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca487000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba5eD44733953d79717F6269357C77718C8Ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000002617500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001406627714f3F17a701f7074a12C02847a5D2Ca487000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d47b22636f6e7472616374223a2230783735366536393666366533313635373536353735363537353635373533393736363137323334373936383634373237353739376136623661363337333638333733343738376136353735363733363633366237393739333633303638373333303736363337313665376137313663333236383731333036633738363333323636222c226d7367223a2265794a6862573931626e51694f694978496e303d222c2266756e6473223a5b5d2c2263616c6c5f616374696f6e223a2263616c6c5f70726f7879227d000000000000000000000000").unwrap().into(),
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

        let json = decode(&encoded).unwrap();

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
                        "_metadata": {
                          "_type": "Solve",
                          "metadata": "0x",
                          "solverAddress": "0xba5ed44733953d79717f6269357c77718c8ba5ed"
                        },
                        "_type": "TokenOrder",
                        "baseAmount": "0x1",
                        "baseToken": "0xba5ed44733953d79717f6269357c77718c8ba5ed",
                        "kind": 3,
                        "metadata": "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                        "quoteAmount": "0x1",
                        "quoteToken": "0x6175",
                        "receiver": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c",
                        "sender": "0x06627714f3f17a701f7074a12c02847a5d2ca487"
                      },
                      "version": 2
                    },
                    {
                      "opcode": 1,
                      "operand": {
                        "_type": "Call",
                        "contractAddress": "0x756e696f6e3132326e79336d6570326c376e687461667077617632793965356a72736c68656b373668736a6c",
                        "contractCalldata": "0x7b22636f6e7472616374223a2230783735366536393666366533313635373536353735363537353635373533393736363137323334373936383634373237353739376136623661363337333638333733343738376136353735363733363633366237393739333633303638373333303736363337313665376137313663333236383731333036633738363333323636222c226d7367223a2265794a6862573931626e51694f694978496e303d222c2266756e6473223a5b5d2c2263616c6c5f616374696f6e223a2263616c6c5f70726f7879227d",
                        "eureka": false,
                        "sender": "0x06627714f3f17a701f7074a12c02847a5d2ca487"
                      },
                      "version": 0
                    }
                  ]
                },
                "version": 0
              },
              "path": "0x0",
              "salt": "0xb515a7377bc2f8914aa44085a2d9f9800dec88985123ea2e1a9be5fa4775ae78"
            })
        );
    }
}
