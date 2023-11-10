// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! `NodeCodec` implementation for Rlp

use std::{borrow::Borrow, marker::PhantomData, ops::Range};

use hash_db::Hasher;
use primitive_types::H256;
use rlp::{DecoderError, Prototype, Rlp, RlpStream};
use sha3::{Digest, Keccak256};
use trie_db::{
    node::{NibbleSlicePlan, NodeHandlePlan, NodePlan, Value, ValuePlan},
    ChildReference, NodeCodec, TrieLayout,
};

#[derive(Default, Clone)]
pub struct EthLayout;

impl TrieLayout for EthLayout {
    const USE_EXTENSION: bool = true;
    const ALLOW_EMPTY: bool = false;
    const MAX_INLINE_VALUE: Option<u32> = None;
    type Hash = KeccakHasher;
    type Codec = RlpNodeCodec<KeccakHasher>;
}

use hash256_std_hasher::Hash256StdHasher;

/// Concrete implementation of Hasher using Keccak 256-bit hashes
#[derive(Debug)]
pub struct KeccakHasher;

impl Hasher for KeccakHasher {
    // TODO: Use `unionlabs::ethereum::H256` here
    type Out = H256;
    type StdHasher = Hash256StdHasher;
    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Self::Out {
        keccak_256(x).into()
    }
}

/// Performs a Keccak-256 hash on the given input.
pub fn keccak_256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Concrete implementation of a `NodeCodec` with Rlp encoding, generic over the `Hasher`
#[derive(Default, Clone)]
pub struct RlpNodeCodec<H: Hasher> {
    mark: PhantomData<H>,
}

const HASHED_NULL_NODE_BYTES: [u8; 32] = [
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];
const HASHED_NULL_NODE: H256 = H256(HASHED_NULL_NODE_BYTES);

/// Encode a partial value with an iterator as input.
fn encode_partial_from_iterator_iter<'a>(
    mut partial: impl Iterator<Item = u8> + 'a,
    odd: bool,
    is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
    let first = if odd { partial.next().unwrap_or(0) } else { 0 };
    encode_partial_inner_iter(first, partial, odd, is_leaf)
}

/// Encode a partial value with an iterator as input.
fn encode_partial_inner_iter<'a>(
    first_byte: u8,
    partial_remaining: impl Iterator<Item = u8> + 'a,
    odd: bool,
    is_leaf: bool,
) -> impl Iterator<Item = u8> + 'a {
    let encoded_type = if is_leaf { 0x20 } else { 0 };
    let first = if odd {
        0x10 + encoded_type + first_byte
    } else {
        encoded_type
    };
    std::iter::once(first).chain(partial_remaining)
}

fn decode_value_range(rlp: Rlp, mut offset: usize) -> Result<Range<usize>, DecoderError> {
    let payload = rlp.payload_info()?;
    offset += payload.header_len;
    Ok(offset..(offset + payload.value_len))
}

fn decode_child_handle_plan<H: Hasher>(
    child_rlp: Rlp,
    mut offset: usize,
) -> Result<NodeHandlePlan, DecoderError> {
    Ok(if child_rlp.is_data() && child_rlp.size() == H::LENGTH {
        let payload = child_rlp.payload_info()?;
        offset += payload.header_len;
        NodeHandlePlan::Hash(offset..(offset + payload.value_len))
    } else {
        NodeHandlePlan::Inline(offset..(offset + child_rlp.as_raw().len()))
    })
}

// NOTE: what we'd really like here is:
// `impl<H: Hasher> NodeCodec<H> for RlpNodeCodec<H> where H::Out: Decodable`
// but due to the current limitations of Rust const evaluation we can't
// do `const HASHED_NULL_NODE: H::Out = H::Out( … … )`. Perhaps one day soon?
impl NodeCodec for RlpNodeCodec<KeccakHasher> {
    type Error = DecoderError;
    type HashOut = <KeccakHasher as Hasher>::Out;

    fn hashed_null_node() -> <KeccakHasher as Hasher>::Out {
        HASHED_NULL_NODE
    }

    fn decode_plan(data: &[u8]) -> Result<NodePlan, Self::Error> {
        let r = Rlp::new(data);
        match r.prototype()? {
            // either leaf or extension - decode first item with NibbleSlice::???
            // and use is_leaf return to figure out which.
            // if leaf, second item is a value (is_data())
            // if extension, second item is a node (either SHA3 to be looked up and
            // fed back into this function or inline RLP which can be fed back into this function).
            Prototype::List(2) => {
                let (partial_rlp, mut partial_offset) = r.at_with_offset(0)?;
                let partial_payload = partial_rlp.payload_info()?;
                partial_offset += partial_payload.header_len;

                let (partial, is_leaf) = if partial_rlp.is_empty() {
                    (
                        NibbleSlicePlan::new(partial_offset..partial_offset, 0),
                        false,
                    )
                } else {
                    let partial_header = partial_rlp.data()?[0];
                    // check leaf bit from header.
                    let is_leaf = partial_header & 32 == 32;
                    // Check the header bit to see if we're dealing with an odd partial (only a nibble of header info)
                    // or an even partial (skip a full byte).
                    let (start, byte_offset) = if partial_header & 16 == 16 {
                        (0, 1)
                    } else {
                        (1, 0)
                    };
                    let range =
                        (partial_offset + start)..(partial_offset + partial_payload.value_len);
                    (NibbleSlicePlan::new(range, byte_offset), is_leaf)
                };

                let (value_rlp, value_offset) = r.at_with_offset(1)?;
                Ok(if is_leaf {
                    let value = decode_value_range(value_rlp, value_offset)?;
                    NodePlan::Leaf {
                        partial,
                        value: ValuePlan::Inline(value),
                    }
                } else {
                    let child = decode_child_handle_plan::<KeccakHasher>(value_rlp, value_offset)?;
                    NodePlan::Extension { partial, child }
                })
            }
            // branch - first 16 are nodes, 17th is a value (or empty).
            Prototype::List(17) => {
                let mut children = [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ];
                for (i, child) in children.iter_mut().enumerate() {
                    let (child_rlp, child_offset) = r.at_with_offset(i)?;
                    if !child_rlp.is_empty() {
                        *child = Some(decode_child_handle_plan::<KeccakHasher>(
                            child_rlp,
                            child_offset,
                        )?);
                    }
                }
                let (value_rlp, value_offset) = r.at_with_offset(16)?;
                let value = if value_rlp.is_empty() {
                    None
                } else {
                    Some(decode_value_range(value_rlp, value_offset)?)
                };
                Ok(NodePlan::Branch {
                    value: value.map(ValuePlan::Inline),
                    children,
                })
            }
            // an empty branch index.
            Prototype::Data(0) => Ok(NodePlan::Empty),
            // something went wrong.
            _ => Err(DecoderError::Custom("Rlp is not valid.")),
        }
    }

    fn is_empty_node(data: &[u8]) -> bool {
        Rlp::new(data).is_empty()
    }

    fn empty_node() -> &'static [u8] {
        &[0x80]
    }

    fn leaf_node(partial: impl Iterator<Item = u8>, _: usize, value: Value) -> Vec<u8> {
        let mut stream = RlpStream::new_list(2);
        stream.append_iter(partial.collect::<Vec<_>>());
        stream.append(&match value {
            Value::Inline(v) => v,
            Value::Node(v) => v,
        });
        stream.out().into()
    }

    fn extension_node(
        partial: impl Iterator<Item = u8>,
        number_nibble: usize,
        child_ref: ChildReference<<KeccakHasher as Hasher>::Out>,
    ) -> Vec<u8> {
        let mut stream = RlpStream::new_list(2);
        stream.append_iter(encode_partial_from_iterator_iter(
            partial,
            number_nibble % 2 > 0,
            false,
        ));
        match child_ref {
            ChildReference::Hash(hash) => stream.append(&hash.as_ref()),
            ChildReference::Inline(inline_data, length) => {
                let bytes = &AsRef::<[u8]>::as_ref(&inline_data)[..length];
                stream.append_raw(bytes, 1)
            }
        };
        stream.out().into()
    }

    fn branch_node(
        children: impl Iterator<
            Item = impl Borrow<Option<ChildReference<<KeccakHasher as Hasher>::Out>>>,
        >,
        value: Option<Value>,
    ) -> Vec<u8> {
        let mut stream = RlpStream::new_list(17);
        for child_ref in children {
            match child_ref.borrow() {
                Some(c) => match c {
                    ChildReference::Hash(h) => stream.append(&h.as_ref()),
                    ChildReference::Inline(inline_data, length) => {
                        let bytes = &AsRef::<[u8]>::as_ref(inline_data)[..*length];
                        stream.append_raw(bytes, 1)
                    }
                },
                None => stream.append_empty_data(),
            };
        }
        if let Some(value) = value {
            stream.append(&match value {
                Value::Inline(v) => v,
                Value::Node(v) => v,
            });
        } else {
            stream.append_empty_data();
        }
        stream.out().into()
    }

    fn branch_node_nibbled(
        _partial: impl Iterator<Item = u8>,
        _number_nibble: usize,
        _children: impl Iterator<
            Item = impl Borrow<Option<ChildReference<<KeccakHasher as Hasher>::Out>>>,
        >,
        _maybe_value: Option<Value>,
    ) -> Vec<u8> {
        unreachable!("This codec is only used with a trie Layout that uses extension node.")
    }
}
