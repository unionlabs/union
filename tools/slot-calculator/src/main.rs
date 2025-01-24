use std::{collections::VecDeque, str::FromStr, vec};

use clap::{Arg, Command};
use slotlib::{MappingKey, Slot};
use typed_arena::Arena;
use unionlabs::primitives::{H256, U256};

// Examples:
// mapping(uint256 => uint256) => ["uint256"]
// mapping(uint256 => uint256[]) => ["uint256", "uint256[]"]
// mapping(uint256 => mapping(uint256 => uint256)) => ["uint256", "uint256"]
// mapping(uint256 => mapping(uint256 => mapping(uint256 => uint256)[])[]) => ["uint256", "mapping[]", "uint256", "mapping[]", "uint256"]
// mapping(uint256 => mapping(uint256 => mapping(uint256 => mapping(uint256 => uint256)[]))[]) => ["uint256", "mapping[]", "uint256", "uint256", "mapping[]", "uint256"]
// mapping(uint256 => mapping(uint256 => uint256)[]) => ["uint256", "mapping[]", "uint256"]
fn parse_layout(layout: &str) -> Result<Vec<String>, String> {
    // 1. Basic validation
    if !layout.starts_with("mapping(") || !layout.ends_with(')') {
        return Err("Invalid layout".to_string());
    }

    // 2. Remove all 'mapping', then strip the first '(' and last ')' character.
    let mut sanitized = layout.replace("mapping", "");
    sanitized = sanitized[1..sanitized.len() - 1].to_string();

    // 3. Split by '=>', trim and remove any leftover '('
    let mut split_by_arrow: Vec<String> = sanitized
        .split("=>")
        .map(|s| s.trim().replace("(", ""))
        .collect();

    // cloning the array so we can iterate indexes from the *original* split_by_arrow
    // while we modify the *current* split_by_arrow in place.
    let original_split = split_by_arrow.clone();

    for (i, element) in original_split.iter().enumerate() {
        let mapping_array_count = element.matches(")[]").count();
        if mapping_array_count == 0 {
            continue;
        }

        // 1) split element by ")[]"
        // 2) skip the first chunk
        // 3) in each chunk, count how many times ')' appears, subtract 1
        // 4) reverse at the end
        let mut skip_indexes: Vec<usize> = element
            .split(")[]")
            .skip(1)
            .map(|chunk| chunk.matches(')').count())
            .collect();
        skip_indexes.reverse();

        // For each occurrence, insert "mapping[]" at index (i - j - skipCount).
        for j in 1..=mapping_array_count {
            let skip_count = skip_indexes[j - 1];
            let insert_index = (i as usize) - (j as usize) - (skip_count as usize);
            split_by_arrow.insert(insert_index as usize, "mapping[]".to_string());
        }
    }

    // 4. Final check on the last element:
    // if it doesn't have "[]" or if it still contains ")[]", pop it
    if let Some(last) = split_by_arrow.last() {
        if !last.contains("[]") || last.contains(")[]") {
            split_by_arrow.pop();
        }
    }

    Ok(split_by_arrow)
}

// MappingKey + U32
#[derive(Debug)]
pub enum KeyTypes<'a> {
    MappingIndex(MappingKey<'a>),
    ArrayIndex(U256),
}

fn parse_keys<'a>(parsed_layout: &Vec<String>, keys: &'a str) -> Vec<KeyTypes<'a>> {
    let split_keys: Vec<&str> = keys.split(" ").collect();
    let mut final_keys: Vec<KeyTypes<'a>> = vec![];
    for (i, parsed_layout_elem) in parsed_layout.iter().enumerate() {
        if (*parsed_layout_elem).contains("[]") {
            final_keys.push(KeyTypes::ArrayIndex(
                split_keys[i].parse::<u32>().unwrap().into(),
            ));
        } else if *parsed_layout_elem == "uint256" {
            final_keys.push(KeyTypes::MappingIndex(MappingKey::Uint256(
                split_keys[i].parse::<u32>().unwrap().into(),
            )));
        } else if *parsed_layout_elem == "string" {
            final_keys.push(KeyTypes::MappingIndex(MappingKey::String(split_keys[i])));
        } else if *parsed_layout_elem == "uint64" {
            final_keys.push(KeyTypes::MappingIndex(MappingKey::Uint64(
                split_keys[i].parse::<u64>().unwrap(),
            )));
        } else if *parsed_layout_elem == "bytes32" {
            final_keys.push(KeyTypes::MappingIndex(MappingKey::Bytes32(
                H256::from_str(split_keys[i]).unwrap(),
            )));
        } else {
            panic!("Unrecognized key!");
        };
    }

    final_keys
}

// ["uint256", "mapping[]", "uint256"]
fn build_slot<'a>(
    parsed_layout: &mut VecDeque<String>,
    parsed_keys: &mut VecDeque<KeyTypes<'a>>,
    arena: &'a Arena<Slot<'a>>,
) -> &'a Slot<'a> {
    if parsed_layout.is_empty() {
        return arena.alloc(Slot::Offset(U256::from(0u32)));
    }

    let layout_part = parsed_layout.pop_front().unwrap();

    if layout_part.contains("[]") {
        let KeyTypes::ArrayIndex(i) = parsed_keys.pop_front().unwrap() else {
            panic!("Expected an array index but got a mapping key!");
        };
        arena.alloc(Slot::Array(
            build_slot(parsed_layout, parsed_keys, arena),
            i,
        ))
    } else {
        let KeyTypes::MappingIndex(mk) = parsed_keys.pop_front().unwrap() else {
            panic!("Expected a mapping key but got an array index!");
        };
        arena.alloc(Slot::Mapping(
            build_slot(parsed_layout, parsed_keys, arena),
            mk,
        ))
    }
}

fn calculate_slot(layout: &str, keys: &str) -> U256 {
    let parsed_layout = parse_layout(layout).unwrap();
    let parsed_keys = parse_keys(&parsed_layout, keys);

    // we need a queue structure for popping first elements more efficiently, therefore we're using VecDeque
    let mut parsed_layout = VecDeque::from(parsed_layout);
    let mut parsed_keys = VecDeque::from(parsed_keys);

    let arena = Arena::new();
    let slot = build_slot(&mut parsed_layout, &mut parsed_keys, &arena);
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

    // Combine user-provided keys into "123 1 100" form:
    let keys_str = keys_collected.join(" ");

    let slot_hex = calculate_slot(layout, &keys_str);

    println!(
        "Calculated storage slot: {}",
        <H256>::new(slot_hex.to_be_bytes())
    );
}

#[test]
fn test_calculate_slot() {
    let layout = "mapping(uint256 => mapping(uint256 => uint256)[])";
    let keys = "100 1 123";

    let slot = calculate_slot(layout, keys);

    assert_eq!(
        <H256>::new(slot.to_be_bytes()),
        <H256>::new(hex_literal::hex!(
            "00a9b48fe93e5d10ebc2d9021d1477088c6292bf047876944343f57fdf3f0467"
        ))
    );
}
