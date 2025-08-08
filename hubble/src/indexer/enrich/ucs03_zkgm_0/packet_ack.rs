use std::{collections::HashMap, fmt::Display};

use alloy_sol_types::SolValue;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};
use sha3::{Digest, Keccak256};

use crate::indexer::enrich::ucs03_zkgm_0::{PacketHash, PacketPathHash};

pub fn decode(
    packet: &[u8],
    ack: Option<&[u8]>,
    packet_hash: &PacketHash,
    mode: Option<&str>,
) -> Result<Value> {
    let packet_value =
        crate::indexer::enrich::ucs03_zkgm_0::packet::decode(packet).context("decode packet")?;

    let ack_value_by_path = match ack {
        Some(ack) => find_acks(
            &mut crate::indexer::enrich::ucs03_zkgm_0::ack::decode(packet, ack)
                .context("decode ack")?,
        )?,
        None => HashMap::new(),
    };

    let mut value = serde_json::to_value(packet_value).context("formatting json")?;

    add_path_ack_and_hash(
        &mut value,
        &ack_value_by_path,
        packet_hash,
        &InstructionPath::root(),
    )?;

    match mode {
        Some("flatten") => Ok(format_flatten(&value)),
        Some("success") => Ok(format_success(&value)),
        Some("tree") => Ok(value),
        Some("all") => Ok(format_all(&value)),
        Some(mode) => Err(anyhow!("invalid mode: {mode}")),
        None => Ok(value),
    }
}

fn find_acks(ack: &mut Value) -> Result<HashMap<InstructionPath, Value>> {
    let mut result = HashMap::new();

    add_acks(ack, &InstructionPath::root(), &mut result)?;

    Ok(result)
}

fn add_acks(
    ack: &mut Value,
    instruction_path: &InstructionPath,
    result: &mut HashMap<InstructionPath, Value>,
) -> Result<()> {
    match ack {
        // If it's an object, check for "_type" and process its fields
        Value::Object(map) => {
            for (_, value) in map.iter_mut() {
                let _ = add_acks(value, instruction_path, result);
            }

            if map.contains_key("_type") || instruction_path.is_root() {
                map.insert(
                    "_index".to_string(),
                    Value::String(instruction_path.to_string()),
                );
                result.insert(instruction_path.clone(), ack.clone());
            }
        }
        // If it's an array, recurse into each element and add index to the path
        Value::Array(arr) => {
            for (index, value) in arr.iter_mut().enumerate() {
                let new_instruction_path = instruction_path.new_with_child(
                    index
                        .try_into()
                        .context(format!("converting index {} from usize to u8", index))?,
                );

                add_acks(value, &new_instruction_path, result)?;
            }
        }
        _ => {} // Do nothing for primitive types
    };

    Ok(())
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct InstructionPath(Vec<u32>);

impl InstructionPath {
    fn root() -> Self {
        Self(vec![])
    }
    fn new_with_child(&self, child: u32) -> Self {
        let mut new_path = self.0.clone();
        new_path.push(child);

        Self(new_path)
    }
    fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for InstructionPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(".")
                .as_str(),
        )
    }
}

fn get_ack_for_path(
    ack_value_by_path: &HashMap<InstructionPath, Value>,
    instruction_path: &InstructionPath,
    expected_type: &String,
) -> Result<Option<Value>> {
    let Some(root) = ack_value_by_path.get(&InstructionPath::root()) else {
        // no root ack => no ack information
        return Ok(None);
    };

    let Some(tag) = root.get("tag") else {
        dbg!(root);
        // no root ack
        return Err(anyhow!("missing root ack tag"));
    };

    let mut ack = if instruction_path.is_root() {
        // root ack is stored in 'innerAck'
        match root.get("innerAck") {
            Some(Value::Object(inner_ack)) => inner_ack.clone(),
            None => Map::new(),
            _ => return Err(anyhow!("expecting object as innerAck")),
        }
    } else {
        // non-root tags are in ack_value_by_path
        match ack_value_by_path.get(instruction_path) {
            Some(Value::Object(ack)) => {
                if let (Some(Value::String(ack_type)), Some(Value::String(ack_index))) =
                    (ack.get("_type"), ack.get("_index"))
                {
                    // expect type and path to align
                    if expected_type != ack_type || &instruction_path.to_string() != ack_index {
                        return Err(anyhow!(
                            "type/index does not align packet type {} <> ack type {} (packet path: {}, ack path: {})",
                            expected_type,
                            ack_type,
                            instruction_path,
                            ack_index
                        ));
                    }

                    // found a matching ack
                    ack.clone()
                } else {
                    return Err(anyhow!(
                        "missing type and/or path in path: {instruction_path}"
                    ));
                }
            }
            None => Map::new(),
            _ => return Err(anyhow!("expecting object as ack")),
        }
    };

    // add tag of the root ack
    ack.remove("_type");
    ack.remove("_index");
    ack.insert("_tag".to_string(), tag.clone());

    Ok(Some(Value::Object(ack)))
}

impl PacketHash {
    fn hash_with_path(&self, path: &InstructionPath) -> PacketPathHash {
        let mut hasher = Keccak256::new();

        // Hash the packet hash
        hasher.update(self.0);

        // Hash the path
        hasher.update(path.0.abi_encode());

        PacketPathHash(hasher.finalize().into())
    }
}

fn add_path_ack_and_hash(
    packet: &mut Value,
    ack_value_by_path: &HashMap<InstructionPath, Value>,
    packet_hash: &PacketHash,
    instruction_path: &InstructionPath,
) -> Result<()> {
    match packet {
        // If it's an object, check for "_type" and process its fields
        Value::Object(map) => {
            for (_, value) in map.iter_mut() {
                add_path_ack_and_hash(value, ack_value_by_path, packet_hash, instruction_path)?;
            }

            if let Some(Value::Object(operand)) = map.get("operand") {
                if let Some(Value::String(packet_type)) = operand.get("_type") {
                    if let Some(ack) =
                        get_ack_for_path(ack_value_by_path, instruction_path, packet_type)?
                    {
                        map.insert("_ack".to_string(), ack);
                    }

                    map.insert(
                        "_index".to_string(),
                        Value::String(instruction_path.to_string()),
                    );
                    map.insert(
                        "_instruction_hash".to_string(),
                        Value::String(packet_hash.hash_with_path(instruction_path).to_0x_hex()),
                    );
                }
            }
        }
        // If it's an array, recurse into each element and add index to the path
        Value::Array(arr) => {
            for (index, value) in arr.iter_mut().enumerate() {
                let new_instruction_path = instruction_path.new_with_child(
                    index
                        .try_into()
                        .context(format!("converting index {} from usize to u8", index))?,
                );

                add_path_ack_and_hash(
                    value,
                    ack_value_by_path,
                    packet_hash,
                    &new_instruction_path,
                )?;
            }
        }
        _ => {} // Do nothing for primitive types
    };
    Ok(())
}

pub fn format_all(tree: &Value) -> Value {
    json!({
      "tree": &tree,
      "flatten": format_flatten(tree),
      "success": format_success(tree),
    })
}

pub fn format_flatten(tree: &Value) -> Value {
    let mut result = Vec::new();

    // collect root attributes, they will be added to each result entry inside a '_root' element
    let root = match tree {
        Value::Object(map) => {
            let mut map = map.clone();
            map.retain(|_, value| !value.is_object());

            Value::Object(map)
        }
        _ => Value::Object(Map::new()),
    };

    flatten_json_tree_recursive(tree, &root, &mut result);

    Value::Array(result)
}

pub fn format_success(tree: &Value) -> Value {
    match tree
        .get("instruction")
        .and_then(|ack| ack.get("_ack"))
        .and_then(|ack| ack.get("_tag"))
        .and_then(Value::as_str)
    {
        Some("0x1") | Some("0x01") => Value::Bool(true), // performance optimatization (expect value to be `0x1`, but want to support `0x001` too)
        Some(tag) if tag.starts_with("0x") => {
            let hex_str = &tag[2..];
            let padded = if hex_str.len() % 2 == 1 {
                format!("0{}", hex_str)
            } else {
                hex_str.to_string()
            };

            match hex::decode(padded) {
                Ok(bytes) => Value::Bool(bytes.len() == 1 && bytes[0] == 1),
                Err(_) => Value::Bool(false),
            }
        }
        Some(_) => Value::Bool(false),
        None => Value::Null,
    }
}

fn flatten_json_tree_recursive(json: &Value, root: &Value, result: &mut Vec<Value>) {
    match json {
        Value::Object(map) => {
            if let Some(Value::Object(operand)) = map.get("operand") {
                if operand.contains_key("_type") {
                    // extend the entry with the 'root' properties
                    let mut entry = map.clone();
                    entry.insert("_root".to_string(), root.clone());

                    // Add the current object to the result
                    result.push(Value::Object(entry));
                }
            }

            for value in map.iter().filter_map(|(key, value)| match key.as_str() {
                "_ack" => None, // we don't want to change the ack
                _ => Some(value),
            }) {
                flatten_json_tree_recursive(value, root, result);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                flatten_json_tree_recursive(value, root, result);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_token_order() {
        let packet = hex::decode("0b00dd4772d3b8ebf5add472a720f986c0846c9b9c1c0ed98f1a011df8486bfc0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002c00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e6831e169d77a861a0e71326afa6d80bcc8bc6aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000014e6831e169d77a861a0e71326afa6d80bcc8bc6aa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000014779877a7b0d9e8603169ddbd7836e478b462478900000000000000000000000000000000000000000000000000000000000000000000000000000000000000044c494e4b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f436861696e4c696e6b20546f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d1b482d1b947a96e96c9b76d15de34f7f70a20a1000000000000000000000000").unwrap();

        let json = decode(&packet, None, &PacketHash([0; 32]), None).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "_index": "",
                "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
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
    fn test_parse_invalid_data() {
        let packet = hex::decode("00").unwrap();

        let result = decode(&packet, None, &PacketHash([0; 32]), None);

        assert!(result.is_err());
    }

    use crate::indexer::enrich::ucs03_zkgm_0::PacketHash;

    #[test]
    fn test_batch_ack() {
        let packet = hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014dc7af843e4eb079cd77ace6774bd71d6b8122f07000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a666163746f72792f756e696f6e31327164766d7732326e37326d656d3079736666336e6c796a32633736637579347836306c75612f636c6f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000148b4bfb23f4d75feef28b4099c0114e5840d14a4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014271126f4f9b36ce16d9e2ef75691485ddce11db60000000000000000000000000000000000000000000000000000000000000000000000000000000000000004cafebabe00000000000000000000000000000000000000000000000000000000").unwrap();

        let ack: &[u8] = &hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let json = decode(&packet, Some(ack), &PacketHash([0; 32]), None).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "_ack": {
                  "_tag": "0x1",
                  "acknowledgements": [
                    {
                      "_index": "0",
                      "_type": "TokenOrder",
                      "fillType": "0xb0cad0",
                      "marketMaker": "0x"
                    },
                    {
                      "_index": "1",
                      "_type": "Call",
                      "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                    }
                  ]
                },
                "_index": "",
                "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                "opcode": 2,
                "operand": {
                  "_type": "Batch",
                  "instructions": [
                    {
                      "_ack": {
                        "_tag": "0x1",
                        "fillType": "0xb0cad0",
                        "marketMaker": "0x"
                      },
                      "_index": "0",
                      "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
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
                      "_ack": {
                        "_tag": "0x1",
                        "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                      },
                      "_index": "1",
                      "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
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

    fn test_batch_ack_without_ack() {
        let packet = hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014dc7af843e4eb079cd77ace6774bd71d6b8122f07000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a666163746f72792f756e696f6e31327164766d7732326e37326d656d3079736666336e6c796a32633736637579347836306c75612f636c6f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000148b4bfb23f4d75feef28b4099c0114e5840d14a4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014271126f4f9b36ce16d9e2ef75691485ddce11db60000000000000000000000000000000000000000000000000000000000000000000000000000000000000004cafebabe00000000000000000000000000000000000000000000000000000000").unwrap();

        let json = decode(&packet, None, &PacketHash([0; 32]), None).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "_index": "",
                "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                "opcode": 2,
                "operand": {
                  "_type": "Batch",
                  "instructions": [
                    {
                      "_index": "0",
                      "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
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
                      "_index": "1",
                      "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
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
    fn test_batch_ack_flatten() {
        let packet = hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014dc7af843e4eb079cd77ace6774bd71d6b8122f07000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a666163746f72792f756e696f6e31327164766d7732326e37326d656d3079736666336e6c796a32633736637579347836306c75612f636c6f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000148b4bfb23f4d75feef28b4099c0114e5840d14a4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014271126f4f9b36ce16d9e2ef75691485ddce11db60000000000000000000000000000000000000000000000000000000000000000000000000000000000000004cafebabe00000000000000000000000000000000000000000000000000000000").unwrap();

        let ack: &[u8] = &hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let json = decode(&packet, Some(ack), &PacketHash([0; 32]), Some("flatten")).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!([
              {
                "_ack": {
                  "_tag": "0x1",
                  "acknowledgements": [
                    {
                      "_index": "0",
                      "_type": "TokenOrder",
                      "fillType": "0xb0cad0",
                      "marketMaker": "0x"
                    },
                    {
                      "_index": "1",
                      "_type": "Call",
                      "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                    }
                  ]
                },
                "_index": "",
                "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                "_root": {
                  "path": "0x0",
                  "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
                },
                "opcode": 2,
                "operand": {
                  "_type": "Batch",
                  "instructions": [
                    {
                      "_ack": {
                        "_tag": "0x1",
                        "fillType": "0xb0cad0",
                        "marketMaker": "0x"
                      },
                      "_index": "0",
                      "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
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
                      "_ack": {
                        "_tag": "0x1",
                        "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                      },
                      "_index": "1",
                      "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
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
              {
                "_ack": {
                  "_tag": "0x1",
                  "fillType": "0xb0cad0",
                  "marketMaker": "0x"
                },
                "_index": "0",
                "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
                "_root": {
                  "path": "0x0",
                  "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
                },
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
                "_ack": {
                  "_tag": "0x1",
                  "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                },
                "_index": "1",
                "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
                "_root": {
                  "path": "0x0",
                  "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
                },
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
            ])
        );
    }

    #[test]
    fn test_batch_ack_all() {
        let packet = hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014dc7af843e4eb079cd77ace6774bd71d6b8122f07000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a666163746f72792f756e696f6e31327164766d7732326e37326d656d3079736666336e6c796a32633736637579347836306c75612f636c6f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000148b4bfb23f4d75feef28b4099c0114e5840d14a4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014271126f4f9b36ce16d9e2ef75691485ddce11db60000000000000000000000000000000000000000000000000000000000000000000000000000000000000004cafebabe00000000000000000000000000000000000000000000000000000000").unwrap();

        let ack: &[u8] = &hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let json = decode(&packet, Some(ack), &PacketHash([0; 32]), Some("all")).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "flatten": [
                {
                  "_ack": {
                    "_tag": "0x1",
                    "acknowledgements": [
                      {
                        "_index": "0",
                        "_type": "TokenOrder",
                        "fillType": "0xb0cad0",
                        "marketMaker": "0x"
                      },
                      {
                        "_index": "1",
                        "_type": "Call",
                        "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                      }
                    ]
                  },
                  "_index": "",
                  "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                  "_root": {
                    "path": "0x0",
                    "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
                  },
                  "opcode": 2,
                  "operand": {
                    "_type": "Batch",
                    "instructions": [
                      {
                        "_ack": {
                          "_tag": "0x1",
                          "fillType": "0xb0cad0",
                          "marketMaker": "0x"
                        },
                        "_index": "0",
                        "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
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
                        "_ack": {
                          "_tag": "0x1",
                          "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                        },
                        "_index": "1",
                        "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
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
                {
                  "_ack": {
                    "_tag": "0x1",
                    "fillType": "0xb0cad0",
                    "marketMaker": "0x"
                  },
                  "_index": "0",
                  "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
                  "_root": {
                    "path": "0x0",
                    "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
                  },
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
                  "_ack": {
                    "_tag": "0x1",
                    "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                  },
                  "_index": "1",
                  "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
                  "_root": {
                    "path": "0x0",
                    "salt": "0x0000000000000000000000000000000000000000000000000000000000000000"
                  },
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
              ],
              "success": true,
              "tree": {
                "instruction": {
                  "_ack": {
                    "_tag": "0x1",
                    "acknowledgements": [
                      {
                        "_index": "0",
                        "_type": "TokenOrder",
                        "fillType": "0xb0cad0",
                        "marketMaker": "0x"
                      },
                      {
                        "_index": "1",
                        "_type": "Call",
                        "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                      }
                    ]
                  },
                  "_index": "",
                  "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                  "opcode": 2,
                  "operand": {
                    "_type": "Batch",
                    "instructions": [
                      {
                        "_ack": {
                          "_tag": "0x1",
                          "fillType": "0xb0cad0",
                          "marketMaker": "0x"
                        },
                        "_index": "0",
                        "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
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
                        "_ack": {
                          "_tag": "0x1",
                          "data": "0x0000000000000000000000000000000000000000000000000000000000000001"
                        },
                        "_index": "1",
                        "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
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
              }
            })
        );
    }

    #[test]
    fn test_batch_ack_success() {
        let packet = hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014dc7af843e4eb079cd77ace6774bd71d6b8122f07000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a666163746f72792f756e696f6e31327164766d7732326e37326d656d3079736666336e6c796a32633736637579347836306c75612f636c6f776e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000148b4bfb23f4d75feef28b4099c0114e5840d14a4700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000014153919669edc8a5d0c8d1e4507c9ce60435a11770000000000000000000000000000000000000000000000000000000000000000000000000000000000000014271126f4f9b36ce16d9e2ef75691485ddce11db60000000000000000000000000000000000000000000000000000000000000000000000000000000000000004cafebabe00000000000000000000000000000000000000000000000000000000").unwrap();

        let ack: &[u8] = &hex::decode("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let json = decode(&packet, Some(ack), &PacketHash([0; 32]), Some("success")).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(json, json!(true));
    }

    #[test]
    fn test_issue() {
        let packet = hex::decode("21DCD61E3C11DB415E36AA1CD285ED7C37A28501C017CDE58F7A2967545A7E270000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000005E0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002A00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001C000000000000000000000000000000000000000000000000000000000000003E8000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000003E8000000000000000000000000000000000000000000000000000000000000001415EE7C367F4232241028C36E720803100757C6E90000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756E696F6E316777716334776774797A6D747A76676D6B326A6565746C6B343570723063646834387A6D357471716D7464736A6675366A6C357338306337336C0000000000000000000000000000000000000000000000000000000000000014F2865969CF99A28BB77E25494FE12D5180FE0EFD00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046D756E6F00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000046D756E6F0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001C00000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000C00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001415EE7C367F4232241028C36E720803100757C6E90000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756E696F6E316777716334776774797A6D747A76676D6B326A6565746C6B343570723063646834387A6D357471716D7464736A6675366A6C357338306337336C00000000000000000000000000000000000000000000000000000000000000807B22626F6E64223A7B22616D6F756E74223A7B22616D6F756E74223A2231323334222C2264656E6F6D223A226D756E6F227D2C2273616C74223A22307832316463643631653363313164623431356533366161316364323835656437633337613238353031633031376364653538663761323936373534356137653237227D7D").unwrap();

        let ack: &[u8] = &hex::decode("000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000").unwrap();

        let json = decode(&packet, Some(ack), &PacketHash([0; 32]), None).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "_ack": {
                  "_tag": "0x0"
                },
                "_index": "",
                "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                "opcode": 2,
                "operand": {
                  "_type": "Batch",
                  "instructions": [
                    {
                      "_ack": {
                        "_tag": "0x0"
                      },
                      "_index": "0",
                      "_instruction_hash": "0x69e40f6af822c360edf576c71482d9bb176e54a4630c0b7ed4194b02df0c30f7",
                      "opcode": 3,
                      "operand": {
                        "_type": "TokenOrder",
                        "baseAmount": "0x3e8",
                        "baseToken": "0xf2865969cf99a28bb77e25494fe12d5180fe0efd",
                        "baseTokenName": "",
                        "baseTokenPath": "0x8",
                        "baseTokenSymbol": "muno",
                        "quoteAmount": "0x3e8",
                        "quoteToken": "0x6d756e6f",
                        "receiver": "0x756e696f6e316777716334776774797a6d747a76676d6b326a6565746c6b343570723063646834387a6d357471716d7464736a6675366a6c357338306337336c",
                        "sender": "0x15ee7c367f4232241028c36e720803100757c6e9"
                      },
                      "version": 0
                    },
                    {
                      "_ack": {
                        "_tag": "0x0"
                      },
                      "_index": "1",
                      "_instruction_hash": "0xdb2bc9ced66bc9a4e1f66497f5ebe43206c2061cab847b3ed3cb165c4ffad3db",
                      "opcode": 1,
                      "operand": {
                        "_type": "Call",
                        "contractAddress": "0x756e696f6e316777716334776774797a6d747a76676d6b326a6565746c6b343570723063646834387a6d357471716d7464736a6675366a6c357338306337336c",
                        "contractCalldata": "0x7b22626f6e64223a7b22616d6f756e74223a7b22616d6f756e74223a2231323334222c2264656e6f6d223a226d756e6f227d2c2273616c74223a22307832316463643631653363313164623431356533366161316364323835656437633337613238353031633031376364653538663761323936373534356137653237227d7d",
                        "eureka": true,
                        "sender": "0x15ee7c367f4232241028c36e720803100757c6e9"
                      },
                      "version": 0
                    }
                  ]
                },
                "version": 0
              },
              "path": "0x0",
              "salt": "0x21dcd61e3c11db415e36aa1cd285ed7c37a28501c017cde58f7a2967545a7e27"
            })
        );
    }

    #[test]
    fn test_token_order_v2_initialize_with_ack() {
        let packet = hex::decode("b4536add4924363adf36c5525508616d702ea6c1e60b6544cd1b542f761a02ab0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000002c756e696f6e316a6b397073796876676b72743263756d7a386579746c6c323234346d326e6e7a34797432673200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed00000000000000000000000000000000000000000000000000000000000000000000000000000000000000046d756e6f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014ba53d2414765913e7b0b47c3ab3fc1e81006e7ba0000000000000000000000000000000000000000000000000000000000000000000000000000000000000020457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280").unwrap();

        let ack: &[u8] = &hex::decode("0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000b0cad000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000").unwrap();

        let json = decode(&packet, Some(ack), &PacketHash([0; 32]), None).unwrap();

        dbg!(serde_json::to_string(&json).unwrap());

        assert_eq!(
            json,
            json!({
              "instruction": {
                "_ack": {
                  "_tag": "0x1",
                  "fillType": "0xb0cad0",
                  "marketMaker": "0x"
                },
                "_index": "",
                "_instruction_hash": "0x02ce32bed30842ace78d6dd11b5c473f23a7fe47341f70996d39525428e373ed",
                "opcode": 3,
                "operand": {
                  "_metadata": {
                    "_type": "Escrow",
                    "data": "0x457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280"
                  },
                  "_type": "TokenOrder",
                  "base_amount": "0x64",
                  "base_token": "0x6d756e6f",
                  "kind": 1,
                  "metadata": "0x457af738e378cb8e744d0dfef10649e127afd4b54efea50c07782464db619280",
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
}
