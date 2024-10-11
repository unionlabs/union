# ADR 050: SIGN_MODE_TEXTUAL

## Changelog

* Dec 06, 2021: Initial Draft.
* Feb 07, 2022: Draft read and concept-ACKed by the Ledger team.
* May 16, 2022: Change status to Accepted.
* Aug 11, 2022: Require signing over tx raw bytes.
* Sep 07, 2022: Add custom `Msg`-renderers.
* Sep 18, 2022: Structured format instead of lines of text
* Nov 23, 2022: Specify CBOR encoding.
* Dec 01, 2022: Link to examples in separate JSON file.
* Dec 06, 2022: Re-ordering of envelope screens.
* Dec 14, 2022: Mention exceptions for invertability.
* Jan 23, 2023: Switch Screen.Text to Title+Content.
* Mar 07, 2023: Change SignDoc from array to struct containing array.
* Mar 20, 2023: Introduce a spec version initialized to 0.

## Status

Accepted. Implementation started. Small value renderers details still need to be polished.

Spec version: 0.

## Abstract

This ADR specifies SIGN_MODE_TEXTUAL, a new string-based sign mode that is targeted at signing with hardware devices.

## Context

Protobuf-based SIGN_MODE_DIRECT was introduced in [ADR-020](./adr-020-protobuf-transaction-encoding.md) and is intended to replace SIGN_MODE_LEGACY_AMINO_JSON in most situations, such as mobile wallets and CLI keyrings. However, the [Ledger](https://www.ledger.com/) hardware wallet is still using SIGN_MODE_LEGACY_AMINO_JSON for displaying the sign bytes to the user. Hardware wallets cannot transition to SIGN_MODE_DIRECT as:

* SIGN_MODE_DIRECT is binary-based and thus not suitable for display to end-users. Technically, hardware wallets could simply display the sign bytes to the user. But this would be considered as blind signing, and is a security concern.
* hardware cannot decode the protobuf sign bytes due to memory constraints, as the Protobuf definitions would need to be embedded on the hardware device.

In an effort to remove Amino from the SDK, a new sign mode needs to be created for hardware devices. [Initial discussions](https://github.com/cosmos/cosmos-sdk/issues/6513) propose a text-based sign mode, which this ADR formally specifies.

## Decision

In SIGN_MODE_TEXTUAL, a transaction is rendered into a textual representation,
which is then sent to a secure device or subsystem for the user to review and sign.
Unlike `SIGN_MODE_DIRECT`, the transmitted data can be simply decoded into legible text
even on devices with limited processing and display.

The textual representation is a sequence of _screens_.
Each screen is meant to be displayed in its entirety (if possible) even on a small device like a Ledger.
A screen is roughly equivalent to a short line of text.
Large screens can be displayed in several pieces,
much as long lines of text are wrapped,
so no hard guidance is given, though 40 characters is a good target.
A screen is used to display a single key/value pair for scalar values
(or composite values with a compact notation, such as `Coins`)
or to introduce or conclude a larger grouping.

The text can contain the full range of Unicode code points, including control characters and nul.
The device is responsible for deciding how to display characters it cannot render natively.
See [annex 2](./adr-050-sign-mode-textual-annex2.md) for guidance.

Screens have a non-negative indentation level to signal composite or nested structures.
Indentation level zero is the top level.
Indentation is displayed via some device-specific mechanism.
Message quotation notation is an appropriate model, such as
leading `>` characters or vertical bars on more capable displays.

Some screens are marked as _expert_ screens,
meant to be displayed only if the viewer chooses to opt in for the extra detail.
Expert screens are meant for information that is rarely useful,
or needs to be present only for signature integrity (see below).

### Invertible Rendering

We require that the rendering of the transaction be invertible:
there must be a parsing function such that for every transaction,
when rendered to the textual representation,
parsing that representation yields a proto message equivalent
to the original under proto equality.

Note that this inverse function does not need to perform correct
parsing or error signaling for the whole domain of textual data.
Merely that the range of valid transactions be invertible under
the composition of rendering and parsing.

Note that the existence of an inverse function ensures that the
rendered text contains the full information of the original transaction,
not a hash or subset.

We make an exception for invertibility for data which are too large to
meaningfully display, such as byte strings longer than 32 bytes. We may then
selectively render them with a cryptographically-strong hash. In these cases,
it is still computationally infeasible to find a different transaction which
has the same rendering. However, we must ensure that the hash computation is
simple enough to be reliably executed independently, so at least the hash is
itself reasonably verifiable when the raw byte string is not.

### Chain State

The rendering function (and parsing function) may depend on the current chain state.
This is useful for reading parameters, such as coin display metadata,
or for reading user-specific preferences such as language or address aliases.
Note that if the observed state changes between signature generation
and the transaction's inclusion in a block, the delivery-time rendering
might differ. If so, the signature will be invalid and the transaction
will be rejected.

### Signature and Security

For security, transaction signatures should have three properties:

1. Given the transaction, signatures, and chain state, it must be possible to validate that the signatures matches the transaction,
to verify that the signers must have known their respective secret keys.

2. It must be computationally infeasible to find a substantially different transaction for which the given signatures are valid, given the same chain state.

3. The user should be able to give informed consent to the signed data via a simple, secure device with limited display capabilities.

The correctness and security of `SIGN_MODE_TEXTUAL` is guaranteed by demonstrating an inverse function from the rendering to transaction protos.
This means that it is impossible for a different protocol buffer message to render to the same text.

### Transaction Hash Malleability

When client software forms a transaction, the "raw" transaction (`TxRaw`) is serialized as a proto
and a hash of the resulting byte sequence is computed.
This is the `TxHash`, and is used by various services to track the submitted transaction through its lifecycle.
Various misbehavior is possible if one can generate a modified transaction with a different TxHash
but for which the signature still checks out.

SIGN_MODE_TEXTUAL prevents this transaction malleability by including the TxHash as an expert screen
in the rendering.

### SignDoc

The SignDoc for `SIGN_MODE_TEXTUAL` is formed from a data structure like:

```go
type Screen struct {
  Title string   // possibly size limited to, advised to 64 characters
  Content string // possibly size limited to, advised to 255 characters
  Indent uint8   // size limited to something small like 16 or 32
  Expert bool
}

type SignDocTextual struct {
  Screens []Screen
}
```

We do not plan to use protobuf serialization to form the sequence of bytes
that will be transmitted and signed, in order to keep the decoder simple.
We will use [CBOR](https://cbor.io) ([RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)) instead.
The encoding is defined by the following CDDL ([RFC 8610](https://www.rfc-editor.org/rfc/rfc8610)):

```
;;; CDDL (RFC 8610) Specification of SignDoc for SIGN_MODE_TEXTUAL.
;;; Must be encoded using CBOR deterministic encoding (RFC 8949, section 4.2.1).

;; A Textual document is a struct containing one field: an array of screens.
sign_doc = {
  screens_key: [* screen],
}

;; The key is an integer to keep the encoding small.
screens_key = 1

;; A screen consists of a text string, an indentation, and the expert flag,
;; represented as an integer-keyed map. All entries are optional
;; and MUST be omitted from the encoding if empty, zero, or false.
;; Text defaults to the empty string, indent defaults to zero,
;; and expert defaults to false.
screen = {
  ? title_key: tstr,
  ? content_key: tstr,
  ? indent_key: uint,
  ? expert_key: bool,
}

;; Keys are small integers to keep the encoding small.
title_key = 1
content_key = 2
indent_key = 3
expert_key = 4
```

Defining the sign_doc as directly an array of screens has also been considered. However, given the possibility of future iterations of this specification, using a single-keyed struct has been chosen over the former proposal, as structs allow for easier backwards-compatibility.

## Details

In the examples that follow, screens will be shown as lines of text,
indentation is indicated with a leading '>',
and expert screens are marked with a leading `*`.

### Encoding of the Transaction Envelope

We define "transaction envelope" as all data in a transaction that is not in the `TxBody.Messages` field. Transaction envelope includes fee, signer infos and memo, but don't include `Msg`s. `//` denotes comments and are not shown on the Ledger device.

```
Chain ID: <string>
Account number: <uint64>
Sequence: <uint64>
Address: <string>
*Public Key: <Any>
This transaction has <int> Message(s)                       // Pluralize "Message" only when int>1
> Message (<int>/<int>): <Any>                              // See value renderers for Any rendering.
End of Message
Memo: <string>                                              // Skipped if no memo set.
Fee: <coins>                                                // See value renderers for coins rendering.
*Fee payer: <string>                                        // Skipped if no fee_payer set.
*Fee granter: <string>                                      // Skipped if no fee_granter set.
Tip: <coins>                                                // Skippted if no tip.
Tipper: <string>
*Gas Limit: <uint64>
*Timeout Height: <uint64>                                   // Skipped if no timeout_height set.
*Other signer: <int> SignerInfo                             // Skipped if the transaction only has 1 signer.
*> Other signer (<int>/<int>): <SignerInfo>
*End of other signers
*Extension options: <int> Any:                              // Skipped if no body extension options
*> Extension options (<int>/<int>): <Any>
*End of extension options
*Non critical extension options: <int> Any:                 // Skipped if no body non critical extension options
*> Non critical extension options (<int>/<int>): <Any>
*End of Non critical extension options
*Hash of raw bytes: <hex_string>                            // Hex encoding of bytes defined, to prevent tx hash malleability.
```

### Encoding of the Transaction Body

Transaction Body is the `Tx.TxBody.Messages` field, which is an array of `Any`s, where each `Any` packs a `sdk.Msg`. Since `sdk.Msg`s are widely used, they have a slightly different encoding than usual array of `Any`s (Protobuf: `repeated google.protobuf.Any`) described in Annex 1.

```
This transaction has <int> message:   // Optional 's' for "message" if there's is >1 sdk.Msgs.
// For each Msg, print the following 2 lines:
Msg (<int>/<int>): <string>           // E.g. Msg (1/2): bank v1beta1 send coins
<value rendering of Msg struct>
End of transaction messages
```

#### Example

Given the following Protobuf message:

```protobuf
message Grant {
  google.protobuf.Any       authorization = 1 [(cosmos_proto.accepts_interface) = "cosmos.authz.v1beta1.Authorization"];
  google.protobuf.Timestamp expiration    = 2 [(gogoproto.stdtime) = true, (gogoproto.nullable) = false];
}

message MsgGrant {
  option (cosmos.msg.v1.signer) = "granter";

  string granter = 1 [(cosmos_proto.scalar) = "cosmos.AddressString"];
  string grantee = 2 [(cosmos_proto.scalar) = "cosmos.AddressString"];
}
```

and a transaction containing 1 such `sdk.Msg`, we get the following encoding:

```
This transaction has 1 message:
Msg (1/1): authz v1beta1 grant
Granter: cosmos1abc...def
Grantee: cosmos1ghi...jkl
End of transaction messages
```

### Custom `Msg` Renderers

Application developers may choose to not follow default renderer value output for their own `Msg`s. In this case, they can implement their own custom `Msg` renderer. This is similar to [EIP4430](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4430.md), where the smart contract developer chooses the description string to be shown to the end user.

This is done by setting the `cosmos.msg.textual.v1.expert_custom_renderer` Protobuf option to a non-empty string. This option CAN ONLY be set on a Protobuf message representing transaction message object (implementing `sdk.Msg` interface).

```protobuf
message MsgFooBar {
  // Optional comments to describe in human-readable language the formatting
  // rules of the custom renderer.
  option (cosmos.msg.textual.v1.expert_custom_renderer) = "<unique algorithm identifier>";

  // proto fields
}
```

When this option is set on a `Msg`, a registered function will transform the `Msg` into an array of one or more strings, which MAY use the key/value format (described in point #3) with the expert field prefix (described in point #5) and arbitrary indentation (point #6). These strings MAY be rendered from a `Msg` field using a default value renderer, or they may be generated from several fields using custom logic.

The `<unique algorithm identifier>` is a string convention chosen by the application developer and is used to identify the custom `Msg` renderer. For example, the documentation or specification of this custom algorithm can reference this identifier. This identifier CAN have a versioned suffix (e.g. `_v1`) to adapt for future changes (which would be consensus-breaking). We also recommend adding Protobuf comments to describe in human language the custom logic used.

Moreover, the renderer must provide 2 functions: one for formatting from Protobuf to string, and one for parsing string to Protobuf. These 2 functions are provided by the application developer. To satisfy point #1, the parse function MUST be the inverse of the formatting function. This property will not be checked by the SDK at runtime. However, we strongly recommend the application developer to include a comprehensive suite in their app repo to test invertibility, as to not introduce security bugs.

### Require signing over the `TxBody` and `AuthInfo` raw bytes

Recall that the transaction bytes merklelized on chain are the Protobuf binary serialization of [TxRaw](https://buf.build/cosmos/cosmos-sdk/docs/main:cosmos.tx.v1beta1#cosmos.tx.v1beta1.TxRaw), which contains the `body_bytes` and `auth_info_bytes`. Moreover, the transaction hash is defined as the SHA256 hash of the `TxRaw` bytes. We require that the user signs over these bytes in SIGN_MODE_TEXTUAL, more specifically over the following string:

```
*Hash of raw bytes: <HEX(sha256(len(body_bytes) ++ body_bytes ++ len(auth_info_bytes) ++ auth_info_bytes))>
```

where:

* `++` denotes concatenation,
* `HEX` is the hexadecimal representation of the bytes, all in capital letters, no `0x` prefix,
* and `len()` is encoded as a Big-Endian uint64.

This is to prevent transaction hash malleability. The point #1 about invertiblity assures that transaction `body` and `auth_info` values are not malleable, but the transaction hash still might be malleable with point #1 only, because the SIGN_MODE_TEXTUAL strings don't follow the byte ordering defined in `body_bytes` and `auth_info_bytes`. Without this hash, a malicious validator or exchange could intercept a transaction, modify its transaction hash _after_ the user signed it using SIGN_MODE_TEXTUAL (by tweaking the byte ordering inside `body_bytes` or `auth_info_bytes`), and then submit it to Tendermint.

By including this hash in the SIGN_MODE_TEXTUAL signing payload, we keep the same level of guarantees as [SIGN_MODE_DIRECT](./adr-020-protobuf-transaction-encoding.md).

These bytes are only shown in expert mode, hence the leading `*`.

## Updates to the current specification

The current specification is not set in stone, and future iterations are to be expected. We distinguish two categories of updates to this specification:

1. Updates that require changes of the hardware device embedded application.
2. Updates that only modify the envelope and the value renderers.

Updates in the 1st category include changes of the `Screen` struct or its corresponding CBOR encoding. This type of updates require a modification of the hardware signer application, to be able to decode and parse the new types. Backwards-compatibility must also be guaranteed, so that the new hardware application works with existing versions of the SDK. These updates require the coordination of multiple parties: SDK developers, hardware application developers (currently: Zondax), and client-side developers (e.g. CosmJS). Furthermore, a new submission of the hardware device application may be necessary, which, depending on the vendor, can take some time. As such, we recommend to avoid this type of updates as much as possible.

Updates in the 2nd category include changes to any of the value renderers or to the transaction envelope. For example, the ordering of fields in the envelope can be swapped, or the timestamp formatting can be modified. Since SIGN_MODE_TEXTUAL sends `Screen`s to the hardware device, this type of change do not need a hardware wallet application update. They are however state-machine-breaking, and must be documented as such. They require the coordination of SDK developers with client-side developers (e.g. CosmJS), so that the updates are released on both sides close to each other in time.

We define a spec version, which is an integer that must be incremented on each update of either category. This spec version will be exposed by the SDK's implementation, and can be communicated to clients. For example, SDK v0.50 might use the spec version 1, and SDK v0.51 might use 2; thanks to this versioning, clients can know how to craft SIGN_MODE_TEXTUAL transactions based on the target SDK version.

The current spec version is defined in the "Status" section, on the top of this document. It is initialized to `0` to allow flexibility in choosing how to define future versions, as it would allow adding a field either in the SignDoc Go struct or in Protobuf in a backwards-compatible way.

## Additional Formatting by the Hardware Device

See [annex 2](./adr-050-sign-mode-textual-annex2.md).

## Examples

1. A minimal MsgSend: [see transaction](https://github.com/cosmos/cosmos-sdk/blob/094abcd393379acbbd043996024d66cd65246fb1/tx/textual/internal/testdata/e2e.json#L2-L70).
2. A transaction with a bit of everything: [see transaction](https://github.com/cosmos/cosmos-sdk/blob/094abcd393379acbbd043996024d66cd65246fb1/tx/textual/internal/testdata/e2e.json#L71-L270).

The examples below are stored in a JSON file with the following fields:

* `proto`: the representation of the transaction in ProtoJSON,
* `screens`: the transaction rendered into SIGN_MODE_TEXTUAL screens,
* `cbor`: the sign bytes of the transaction, which is the CBOR encoding of the screens.

## Consequences

### Backwards Compatibility

SIGN_MODE_TEXTUAL is purely additive, and doesn't break any backwards compatibility with other sign modes.

### Positive

* Human-friendly way of signing in hardware devices.
* Once SIGN_MODE_TEXTUAL is shipped, SIGN_MODE_LEGACY_AMINO_JSON can be deprecated and removed. On the longer term, once the ecosystem has totally migrated, Amino can be totally removed.

### Negative

* Some fields are still encoded in non-human-readable ways, such as public keys in hexadecimal.
* New ledger app needs to be released, still unclear

### Neutral

* If the transaction is complex, the string array can be arbitrarily long, and some users might just skip some screens and blind sign.

## Further Discussions

* Some details on value renderers need to be polished, see [Annex 1](./adr-050-sign-mode-textual-annex1.md).
* Are ledger apps able to support both SIGN_MODE_LEGACY_AMINO_JSON and SIGN_MODE_TEXTUAL at the same time?
* Open question: should we add a Protobuf field option to allow app developers to overwrite the textual representation of certain Protobuf fields and message? This would be similar to Ethereum's [EIP4430](https://github.com/ethereum/EIPs/pull/4430), where the contract developer decides on the textual representation.
* Internationalization.

## References

* [Annex 1](./adr-050-sign-mode-textual-annex1.md)

* Initial discussion: https://github.com/cosmos/cosmos-sdk/issues/6513
* Living document used in the working group: https://hackmd.io/fsZAO-TfT0CKmLDtfMcKeA?both
* Working group meeting notes: https://hackmd.io/7RkGfv_rQAaZzEigUYhcXw
* Ethereum's "Described Transactions" https://github.com/ethereum/EIPs/pull/4430
