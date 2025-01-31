use clap::{Arg, Command};
use proc_macro2::TokenStream;
use slotlib::{MappingKey, Slot};
use std::{collections::VecDeque, str::FromStr};
use syn_solidity::{parse2, Item, Type};
use typed_arena::Arena;
use unionlabs::primitives::{H256, U256};

// Examples:
// mapping(uint256 => uint256)
// uint256[] => ["uint256[]"]
// mapping(uint256 => uint256[])
// mapping(uint256 => mapping(uint256 => uint256))
// mapping(uint256 => mapping(uint256 => mapping(uint256 => uint256)[])[])
// mapping(uint256 => mapping(uint256 => mapping(uint256 => mapping(uint256 => uint256)[]))[])
// mapping(uint256 => mapping(uint256 => uint256)[])
fn parse_layout(layout: &mut String) -> Result<Type, String> {
    // Check if the layout already includes a visibility modifier and a variable name
    if !layout.contains("public") && !layout.contains("internal") && !layout.contains("private") {
        layout.push_str(" public dummyName;");
    }

    let parsed_layout = layout
        .parse::<TokenStream>()
        .map_err(|_| "Failed to parse layout".to_string())?;
    let parsed_layout = parse2(parsed_layout).map_err(|e| e.to_string())?;

    match &parsed_layout.items[0] {
        Item::Variable(var) => match &var.ty {
            Type::Mapping(map) => Ok(Type::Mapping(map.clone())),
            Type::Array(arr) => Ok(Type::Array(arr.clone())),
            _ => return Err("Unsupported type".to_string()),
        },
        _ => return Err("Unsupported item".to_string()),
    }
}

fn parse_mapping_key<'a>(key_type: &'a Type, key: &'a str) -> MappingKey<'a> {
    match key_type {
        Type::Uint(_, size) => {
            let size = size.and_then(|s| Some(s.get())).unwrap_or(256);
            match size {
                256 => MappingKey::Uint256(U256::from(
                    key.parse::<u64>().expect("Invalid uint256 key"),
                )),
                64 => MappingKey::Uint64(key.parse::<u64>().expect("Invalid uint64 key")),
                _ => panic!("Unsupported uint size: {}", size),
            }
        }
        Type::FixedBytes(_, size) => {
            let size = size.get();
            match size {
                32 => MappingKey::Bytes32(H256::from_str(key).expect("Invalid bytes32 key")),
                _ => panic!("Unsupported bytes size: {}", size),
            }
        }
        Type::String(_) => MappingKey::String(key),
        _ => panic!("Unsupported mapping key type"),
    }
}

fn build_slot<'a>(
    parsed_layout: &'a Type,
    keys: &mut VecDeque<&'a str>,
    arena: &'a Arena<Slot<'a>>,
) -> &'a Slot<'a> {
    let key: &str = match keys.pop_front() {
        Some(k) => k,
        None => return arena.alloc(Slot::Offset(U256::from(0u32))),
    };
    match parsed_layout {
        Type::Mapping(mapping) => arena.alloc(Slot::Mapping(
            build_slot(mapping.value.as_ref(), keys, arena),
            parse_mapping_key(mapping.key.as_ref(), key),
        )),
        Type::Array(arr) => arena.alloc(Slot::Array(
            build_slot(arr.ty.as_ref(), keys, arena),
            U256::from(key.parse::<u64>().expect("Invalid array index")),
        )),
        _ => panic!("Unsupported layout type or wrong key count."),
    }
}

fn calculate_slot(layout: &str, keys: &str) -> U256 {
    let parsed_layout = parse_layout(&mut layout.to_string()).unwrap();
    let mut split_keys: VecDeque<&str> = keys.split(" ").collect();

    let arena = Arena::new();
    let slot = build_slot(&parsed_layout, &mut split_keys, &arena);
    if split_keys.len() != 0 {
        eprintln!(
            "Warning: Unused keys: {:?}. The calculated slot might be wrong. Please check the layout and keys you provided.",
            split_keys
        );
    }
    slot.slot()
}

fn main() {
    let matches = Command::new("Slot Calculator (post-order)")
        .version("1.0")
        .about("Calculates Solidity storage slots for various layouts in a post-order manner")
        .arg(
            Arg::new("layout")
                .long("layout")
                .value_name("LAYOUT")
                .required(true)
                .help("e.g. 'mapping(uint256 => mapping(uint256 => uint256)[])'"),
        )
        .arg(
            Arg::new("keys")
                .long("keys")
                .value_name("KEYS")
                .required(true)
                .num_args(1..) // Accept one or more
                .help("The keys in the order that matches your snippet, e.g. 123 1 100"),
        )
        .get_matches();

    let layout = matches
        .get_one::<String>("layout")
        .expect("Missing --layout");
    let keys_collected: Vec<String> = matches
        .get_many::<String>("keys")
        .expect("Missing --keys")
        .map(|s| s.to_string())
        .collect();

    let keys_str = keys_collected.join(" ");

    let slot_hex = calculate_slot(layout, &keys_str);

    println!(
        "Calculated storage slot: {}",
        <H256>::new(slot_hex.to_be_bytes())
    );
}

#[test]
fn test_calculate_slot() {
    let layout = "mapping(uint256 => mapping(uint256 => uint256)[]) public test;";
    let keys = "100 1 123";

    let slot = calculate_slot(layout, keys);

    assert_eq!(
        <H256>::new(slot.to_be_bytes()),
        <H256>::new(hex_literal::hex!(
            "00a9b48fe93e5d10ebc2d9021d1477088c6292bf047876944343f57fdf3f0467"
        ))
    );
}
