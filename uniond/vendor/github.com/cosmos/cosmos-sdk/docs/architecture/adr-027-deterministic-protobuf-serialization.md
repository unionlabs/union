# ADR 027: Deterministic Protobuf Serialization

## Changelog

* 2020-08-07: Initial Draft
* 2020-09-01: Further clarify rules

## Status

Proposed

## Abstract

Fully deterministic structure serialization, which works across many languages and clients,
is needed when signing messages. We need to be sure that whenever we serialize
a data structure, no matter in which supported language, the raw bytes
will stay the same.
[Protobuf](https://developers.google.com/protocol-buffers/docs/proto3)
serialization is not bijective (i.e. there exist a practically unlimited number of
valid binary representations for a given protobuf document)<sup>1</sup>.

This document describes a deterministic serialization scheme for
a subset of protobuf documents, that covers this use case but can be reused in
other cases as well.

### Context

For signature verification in Cosmos SDK, the signer and verifier need to agree on
the same serialization of a `SignDoc` as defined in
[ADR-020](./adr-020-protobuf-transaction-encoding.md) without transmitting the
serialization.

Currently, for block signatures we are using a workaround: we create a new [TxRaw](https://github.com/cosmos/cosmos-sdk/blob/9e85e81e0e8140067dd893421290c191529c148c/proto/cosmos/tx/v1beta1/tx.proto#L30)
instance (as defined in [adr-020-protobuf-transaction-encoding](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-020-protobuf-transaction-encoding.md#transactions))
by converting all [Tx](https://github.com/cosmos/cosmos-sdk/blob/9e85e81e0e8140067dd893421290c191529c148c/proto/cosmos/tx/v1beta1/tx.proto#L13)
fields to bytes on the client side. This adds an additional manual
step when sending and signing transactions.

### Decision

The following encoding scheme is to be used by other ADRs,
and in particular for `SignDoc` serialization.

## Specification

### Scope

This ADR defines a protobuf3 serializer. The output is a valid protobuf
serialization, such that every protobuf parser can parse it.

No maps are supported in version 1 due to the complexity of defining a
deterministic serialization. This might change in future. Implementations must
reject documents containing maps as invalid input.

### Background - Protobuf3 Encoding

Most numeric types in protobuf3 are encoded as
[varints](https://developers.google.com/protocol-buffers/docs/encoding#varints).
Varints are at most 10 bytes, and since each varint byte has 7 bits of data,
varints are a representation of `uint70` (70-bit unsigned integer). When
encoding, numeric values are casted from their base type to `uint70`, and when
decoding, the parsed `uint70` is casted to the appropriate numeric type.

The maximum valid value for a varint that complies with protobuf3 is
`FF FF FF FF FF FF FF FF FF 7F` (i.e. `2**70 -1`). If the field type is
`{,u,s}int64`, the highest 6 bits of the 70 are dropped during decoding,
introducing 6 bits of malleability. If the field type is `{,u,s}int32`, the
highest 38 bits of the 70 are dropped during decoding, introducing 38 bits of
malleability.

Among other sources of non-determinism, this ADR eliminates the possibility of
encoding malleability.

### Serialization rules

The serialization is based on the
[protobuf3 encoding](https://developers.google.com/protocol-buffers/docs/encoding)
with the following additions:

1. Fields must be serialized only once in ascending order
2. Extra fields or any extra data must not be added
3. [Default values](https://developers.google.com/protocol-buffers/docs/proto3#default)
   must be omitted
4. `repeated` fields of scalar numeric types must use
   [packed encoding](https://developers.google.com/protocol-buffers/docs/encoding#packed)
5. Varint encoding must not be longer than needed:
    * No trailing zero bytes (in little endian, i.e. no leading zeroes in big
      endian). Per rule 3 above, the default value of `0` must be omitted, so
      this rule does not apply in such cases.
    * The maximum value for a varint must be `FF FF FF FF FF FF FF FF FF 01`.
      In other words, when decoded, the highest 6 bits of the 70-bit unsigned
      integer must be `0`. (10-byte varints are 10 groups of 7 bits, i.e.
      70 bits, of which only the lowest 70-6=64 are useful.)
    * The maximum value for 32-bit values in varint encoding must be `FF FF FF FF 0F`
      with one exception (below). In other words, when decoded, the highest 38
      bits of the 70-bit unsigned integer must be `0`.
        * The one exception to the above is _negative_ `int32`, which must be
          encoded using the full 10 bytes for sign extension<sup>2</sup>.
    * The maximum value for Boolean values in varint encoding must be `01` (i.e.
      it must be `0` or `1`). Per rule 3 above, the default value of `0` must
      be omitted, so if a Boolean is included it must have a value of `1`.

While rule number 1. and 2. should be pretty straight forward and describe the
default behavior of all protobuf encoders the author is aware of, the 3rd rule
is more interesting. After a protobuf3 deserialization you cannot differentiate
between unset fields and fields set to the default value<sup>3</sup>. At
serialization level however, it is possible to set the fields with an empty
value or omitting them entirely. This is a significant difference to e.g. JSON
where a property can be empty (`""`, `0`), `null` or undefined, leading to 3
different documents.

Omitting fields set to default values is valid because the parser must assign
the default value to fields missing in the serialization<sup>4</sup>. For scalar
types, omitting defaults is required by the spec<sup>5</sup>. For `repeated`
fields, not serializing them is the only way to express empty lists. Enums must
have a first element of numeric value 0, which is the default<sup>6</sup>. And
message fields default to unset<sup>7</sup>.

Omitting defaults allows for some amount of forward compatibility: users of
newer versions of a protobuf schema produce the same serialization as users of
older versions as long as newly added fields are not used (i.e. set to their
default value).

### Implementation

There are three main implementation strategies, ordered from the least to the
most custom development:

* **Use a protobuf serializer that follows the above rules by default.** E.g.
  [gogoproto](https://pkg.go.dev/github.com/cosmos/gogoproto/gogoproto) is known to
  be compliant by in most cases, but not when certain annotations such as
  `nullable = false` are used. It might also be an option to configure an
  existing serializer accordingly.
* **Normalize default values before encoding them.** If your serializer follows
  rule 1. and 2. and allows you to explicitly unset fields for serialization,
  you can normalize default values to unset. This can be done when working with
  [protobuf.js](https://www.npmjs.com/package/protobufjs):

  ```js
  const bytes = SignDoc.encode({
    bodyBytes: body.length > 0 ? body : null, // normalize empty bytes to unset
    authInfoBytes: authInfo.length > 0 ? authInfo : null, // normalize empty bytes to unset
    chainId: chainId || null, // normalize "" to unset
    accountNumber: accountNumber || null, // normalize 0 to unset
    accountSequence: accountSequence || null, // normalize 0 to unset
  }).finish();
  ```

* **Use a hand-written serializer for the types you need.** If none of the above
  ways works for you, you can write a serializer yourself. For SignDoc this
  would look something like this in Go, building on existing protobuf utilities:

  ```go
  if !signDoc.body_bytes.empty() {
      buf.WriteUVarInt64(0xA) // wire type and field number for body_bytes
      buf.WriteUVarInt64(signDoc.body_bytes.length())
      buf.WriteBytes(signDoc.body_bytes)
  }

  if !signDoc.auth_info.empty() {
      buf.WriteUVarInt64(0x12) // wire type and field number for auth_info
      buf.WriteUVarInt64(signDoc.auth_info.length())
      buf.WriteBytes(signDoc.auth_info)
  }

  if !signDoc.chain_id.empty() {
      buf.WriteUVarInt64(0x1a) // wire type and field number for chain_id
      buf.WriteUVarInt64(signDoc.chain_id.length())
      buf.WriteBytes(signDoc.chain_id)
  }

  if signDoc.account_number != 0 {
      buf.WriteUVarInt64(0x20) // wire type and field number for account_number
      buf.WriteUVarInt(signDoc.account_number)
  }

  if signDoc.account_sequence != 0 {
      buf.WriteUVarInt64(0x28) // wire type and field number for account_sequence
      buf.WriteUVarInt(signDoc.account_sequence)
  }
  ```

### Test vectors

Given the protobuf definition `Article.proto`

```protobuf
package blog;
syntax = "proto3";

enum Type {
  UNSPECIFIED = 0;
  IMAGES = 1;
  NEWS = 2;
};

enum Review {
  UNSPECIFIED = 0;
  ACCEPTED = 1;
  REJECTED = 2;
};

message Article {
  string title = 1;
  string description = 2;
  uint64 created = 3;
  uint64 updated = 4;
  bool public = 5;
  bool promoted = 6;
  Type type = 7;
  Review review = 8;
  repeated string comments = 9;
  repeated string backlinks = 10;
};
```

serializing the values

```yaml
title: "The world needs change 🌳"
description: ""
created: 1596806111080
updated: 0
public: true
promoted: false
type: Type.NEWS
review: Review.UNSPECIFIED
comments: ["Nice one", "Thank you"]
backlinks: []
```

must result in the serialization

```text
0a1b54686520776f726c64206e65656473206368616e676520f09f8cb318e8bebec8bc2e280138024a084e696365206f6e654a095468616e6b20796f75
```

When inspecting the serialized document, you see that every second field is
omitted:

```shell
$ echo 0a1b54686520776f726c64206e65656473206368616e676520f09f8cb318e8bebec8bc2e280138024a084e696365206f6e654a095468616e6b20796f75 | xxd -r -p | protoc --decode_raw
1: "The world needs change \360\237\214\263"
3: 1596806111080
5: 1
7: 2
9: "Nice one"
9: "Thank you"
```

## Consequences

Having such an encoding available allows us to get deterministic serialization
for all protobuf documents we need in the context of Cosmos SDK signing.

### Positive

* Well defined rules that can be verified independent of a reference
  implementation
* Simple enough to keep the barrier to implement transaction signing low
* It allows us to continue to use 0 and other empty values in SignDoc, avoiding
  the need to work around 0 sequences. This does not imply the change from
  https://github.com/cosmos/cosmos-sdk/pull/6949 should not be merged, but not
  too important anymore.

### Negative

* When implementing transaction signing, the encoding rules above must be
  understood and implemented.
* The need for rule number 3. adds some complexity to implementations.
* Some data structures may require custom code for serialization. Thus
  the code is not very portable - it will require additional work for each
  client implementing serialization to properly handle custom data structures.

### Neutral

### Usage in Cosmos SDK

For the reasons mentioned above ("Negative" section) we prefer to keep workarounds
for shared data structure. Example: the aforementioned `TxRaw` is using raw bytes
as a workaround. This allows them to use any valid Protobuf library without
the need of implementing a custom serializer that adheres to this standard (and related risks of bugs).

## References

* <sup>1</sup> _When a message is serialized, there is no guaranteed order for
  how its known or unknown fields should be written. Serialization order is an
  implementation detail and the details of any particular implementation may
  change in the future. Therefore, protocol buffer parsers must be able to parse
  fields in any order._ from
  https://developers.google.com/protocol-buffers/docs/encoding#order
* <sup>2</sup> https://developers.google.com/protocol-buffers/docs/encoding#signed_integers
* <sup>3</sup> _Note that for scalar message fields, once a message is parsed
  there's no way of telling whether a field was explicitly set to the default
  value (for example whether a boolean was set to false) or just not set at all:
  you should bear this in mind when defining your message types. For example,
  don't have a boolean that switches on some behavior when set to false if you
  don't want that behavior to also happen by default._ from
  https://developers.google.com/protocol-buffers/docs/proto3#default
* <sup>4</sup> _When a message is parsed, if the encoded message does not
  contain a particular singular element, the corresponding field in the parsed
  object is set to the default value for that field._ from
  https://developers.google.com/protocol-buffers/docs/proto3#default
* <sup>5</sup> _Also note that if a scalar message field is set to its default,
  the value will not be serialized on the wire._ from
  https://developers.google.com/protocol-buffers/docs/proto3#default
* <sup>6</sup> _For enums, the default value is the first defined enum value,
  which must be 0._ from
  https://developers.google.com/protocol-buffers/docs/proto3#default
* <sup>7</sup> _For message fields, the field is not set. Its exact value is
  language-dependent._ from
  https://developers.google.com/protocol-buffers/docs/proto3#default
* Encoding rules and parts of the reasoning taken from
  [canonical-proto3 Aaron Craelius](https://github.com/regen-network/canonical-proto3)
