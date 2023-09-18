# ADR 050: SIGN_MODE_TEXTUAL

## Changelog

- Dec 06, 2021: Initial Draft.
- Feb 07, 2022: Draft read and concept-ACKed by the Ledger team.
- May 16, 2022: Change status to Accepted.
- Aug 11, 2022: Require signing over tx raw bytes.
- Sep 07, 2022: Add custom `Msg`-renderers.
- Sep 18, 2022: Structured format instead of lines of text

## Status

Accepted. Implementation started. Small value renderers details still need to be polished.

## Abstract

This ADR specifies SIGN_MODE_TEXTUAL, a new string-based sign mode that is targetted at signing with hardware devices.

## Context

Protobuf-based SIGN_MODE_DIRECT was introduced in [ADR-020](./adr-020-protobuf-transaction-encoding.md) and is intended to replace SIGN_MODE_LEGACY_AMINO_JSON in most situations, such as mobile wallets and CLI keyrings. However, the [Ledger](https://www.ledger.com/) hardware wallet is still using SIGN_MODE_LEGACY_AMINO_JSON for displaying the sign bytes to the user. Hardware wallets cannot transition to SIGN_MODE_DIRECT as:

- SIGN_MODE_DIRECT is binary-based and thus not suitable for display to end-users. Technically, hardware wallets could simply display the sign bytes to the user. But this would be considered as blind signing, and is a security concern.
- hardware cannot decode the protobuf sign bytes due to memory constraints, as the Protobuf definitions would need to be embedded on the hardware device.

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
parsing that representation yeilds a proto message equivalent
to the original under proto equality.

Note that this inverse function does not need to perform correct
parsing or error signaling for the whole domain of textual data.
Merely that the range of valid transactions be invertible under
the composition of rendering and parsing.

Note that the existence of an inverse function ensures that the
rendered text contains the full information of the original transaction,
not a hash or subset.

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

```
type Screen struct {
  Text string text  // possibly size limited to, e.g. 255 characters
  Indent uint8  // size limited to something small like 16 or 32
  Expert bool
}

type SignDocTextual = []Screen
```

We do not plan to use protobuf serialization to form the sequence of bytes
that will be tranmitted and signed, in order to keep the decoder simple.
We will use [CBOR](https://cbor.io) ([RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)) instead.

TODO: specify the details of the CBOR encoding.

## Details

In the examples that follow, screens will be shown as lines of text,
indentation is indicated with a leading '>',
and expert screens are marked with a leading `*`.

### Encoding of the Transaction Envelope

We define "transaction envelope" as all data in a transaction that is not in the `TxBody.Messages` field. Transaction envelope includes fee, signer infos and memo, but don't include `Msg`s. `//` denotes comments and are not shown on the Ledger device.

```
Chain ID: <string>
Account number: <uint64>
*Public Key: <hex_string>
Sequence: <uint64>
<TxBody>                                                    // See #8.
Fee: <coins>                                                // See value renderers for coins encoding.
*Fee payer: <string>                                        // Skipped if no fee_payer set
*Fee granter: <string>                                      // Skipped if no fee_granter set
Memo: <string>                                              // Skipped if no memo set
*Gas Limit: <uint64>
*Timeout Height:  <uint64>                                  // Skipped if no timeout_height set
Tipper: <string>                                            // If there's a tip
Tip: <string>
*This transaction has <int> body extension:                 // Skipped if no body extension options
*<repeated Any>
*This transaction has <int> body non-critical extensions:   // Skipped if no body non-critical extension options
*<repeated Any>                                             // See value renderers for Any and array encoding.
*This transaction has <int> body auth info extensions:      // Skipped if no auth info extension options
*<repeated Any>
*This transaction has <int> other signers:                  // Skipped if there is only one signer
*Signer (<int>/<int>):
*Public Key: <hex_string>
*Sequence: <uint64>
*End of other signers
*Hash of raw bytes: <hex_string>                            // Hex encoding of bytes defined in #10, to prevent tx hash malleability.
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
  option (cosmos.msg.v1.textual.type_url) = "authz v1beta1 grant";

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

This is done by setting the `cosmos.msg.v1.textual.expert_custom_renderer` Protobuf option to a non-empty string. This option CAN ONLY be set on a Protobuf message representing transaction message object (implementing `sdk.Msg` interface).

```protobuf
message MsgFooBar {
  // Optional comments to describe in human-readable language the formatting
  // rules of the custom renderer.
  option (cosmos.msg.v1.textual.expert_custom_renderer) = "<unique algorithm identifier>";

  // proto fields
}
```

When this option is set on a `Msg`, a registered function will transform the `Msg` into an array of one or more strings, which MAY use the key/value format (described in point #3) with the expert field prefix (described in point #5) and arbitrary indentation (point #6). These strings MAY be rendered from a `Msg` field using a default value renderer, or they may be generated from several fields using custom logic.

The `<unique algorithm identifier>` is a string convention chosen by the application developer and is used to identify the custom `Msg` renderer. For example, the documentation or specification of this custom algorithm can reference this identifier. This identifier CAN have a versioned suffix (e.g. `_v1`) to adapt for future changes (which would be consensus-breaking). We also recommend adding Protobuf comments to describe in human language the custom logic used.

Moreover, the renderer must provide 2 functions: one for formatting from Protobuf to string, and one for parsing string to Protobuf. These 2 functions are provided by the application developer. To satisfy point #1, the parse function MUST be the inverse of the formatting function. This property will not be checked by the SDK at runtime. However, we strongly recommend the application developer to include a comprehensive suite in their app repo to test invertibility, as to not introduce security bugs.

### Require signing over the `TxBody` and `AuthInfo` raw bytes

Recall that the transaction bytes merklelized on chain are the Protobuf binary serialization of [TxRaw](hhttps://buf.build/cosmos/cosmos-sdk/docs/main:cosmos.tx.v1beta1#cosmos.tx.v1beta1.TxRaw), which contains the `body_bytes` and `auth_info_bytes`. Moreover, the transaction hash is defined as the SHA256 hash of the `TxRaw` bytes. We require that the user signs over these bytes in SIGN_MODE_TEXTUAL, more specifically over the following string:

```
*Hash of raw bytes: <HEX(sha256(len(body_bytes) ++ body_bytes ++ len(auth_info_bytes) ++ auth_info_bytes))>
```

where:
- `++` denotes concatenation,
- `HEX` is the hexadecimal representation of the bytes, all in capital letters, no `0x` prefix,
- and `len()` is encoded as a Big-Endian uint64.

This is to prevent transaction hash malleability. The point #1 about invertiblity assures that transaction `body` and `auth_info` values are not malleable, but the transaction hash still might be malleable with point #1 only, because the SIGN_MODE_TEXTUAL strings don't follow the byte ordering defined in `body_bytes` and `auth_info_bytes`. Without this hash, a malicious validator or exchange could intercept a transaction, modify its transaction hash _after_ the user signed it using SIGN_MODE_TEXTUAL (by tweaking the byte ordering inside `body_bytes` or `auth_info_bytes`), and then submit it to Tendermint.

By including this hash in the SIGN_MODE_TEXTUAL signing payload, we keep the same level of guarantees as [SIGN_MODE_DIRECT](./adr-020-protobuf-transaction-encoding.md).

These bytes are only shown in expert mode, hence the leading `*`.

## Additional Formatting by the Hardware Device

See [annex 2](./adr-050-sign-mode-textual-annex2.md).

## Examples

#### Example 1: Simple `MsgSend`

JSON:

```json
{
  "body": {
    "messages": [
      {
        "@type": "/cosmos.bank.v1beta1.MsgSend",
        "from": "cosmos1...abc",
        "to": "cosmos1...def",
        "amount": [
          {
            "denom": "uatom",
            "amount": 10000000
          }
        ]
      }
    ]
  },
  "auth_info": {
    "signer_infos": [
      {
        "public_key": "iQ...==",
        "mode_info": { "single": { "mode": "SIGN_MODE_TEXTUAL" } },
        "sequence": 2
      }
    ],
    "fee": {
      "amount": [
        {
          "denom": "atom",
          "amount": 0.002
        }
      ],
      "gas_limit": 100000
    }
  },
  // Additional SignerData.
  "chain_id": "simapp-1",
  "account_number": 10
}
```

SIGN_MODE_TEXTUAL:

```
Chain ID: simapp-1
Account number: 10
*Public Key: iQ...==        // Hex pubkey
Sequence: 2
This transaction has 1 message:
Message (1/1): bank v1beta1 send coins
From: cosmos1...abc
To: cosmos1...def
Amount: 10 atom            // Conversion from uatom to atom using value renderers
End of transaction messages
Fee: 0.002 atom
*Gas: 100'000
*Hash of raw bytes: <hex_string>
```

#### Example 2: Multi-Msg Transaction with 3 signers

#### Example 3: Legacy Multisig

#### Example 4: Fee Payer with Tips

```json
{
  "body": {
    "messages": [
      {
        "@type": "/cosmos.bank.v1beta1.MsgSend",
        "from": "cosmos1...tipper",
        "to": "cosmos1...abc",
        "amount": [
          {
            "denom": "uatom",
            "amount": 10000000
          }
        ]
      }
    ]
  },
  "auth_info": {
    "signer_infos": [
      {
        "public_key": "iQ...==",
        "mode_info": { "single": { "mode": "SIGN_MODE_DIRECT_AUX" } },
        "sequence": 42
      },
      {
        "public_key": "iR...==",
        "mode_info": { "single": { "mode": "SIGN_MODE_TEXTUAL" } },
        "sequence": 2
      }
    ],
    "fee": {
      "amount": [
        {
          "denom": "atom",
          "amount": 0.002
        }
      ],
      "gas_limit": 100000,
      "payer": "cosmos1...feepayer"
    },
    "tip": {
      "amount": [
        {
          "denom": "ibc/CDC4587874B85BEA4FCEC3CEA5A1195139799A1FEE711A07D972537E18FDA39D",
          "amount": 200
        }
      ],
      "tipper": "cosmos1...tipper"
    }
  },
  // Additional SignerData.
  "chain_id": "simapp-1",
  "account_number": 10
}
```

SIGN_MODE_TEXTUAL for the feepayer:

```
Chain ID: simapp-1
Account number: 10
*Public Key: iR...==
Sequence: 2
This transaction has 1 message:
Message (1/1): bank v1beta1 send coins
From: cosmos1...abc
To: cosmos1...def
Amount: 10 atom
End of transaction messages
Fee: 0.002 atom
Fee Payer: cosmos1...feepayer
Tipper: cosmos1...tipper
Tip: 200 ibc/CDC4587874B85BEA4FCEC3CEA5A1195139799A1FEE711A07D972537E18FDA39D
*Gas: 100'000
*This transaction has 1 other signer:
*Signer (1/2):
*Public Key: iQ...==
*Sign mode: Direct Aux
*Sequence: 42
*End of other signers
*Hash of raw bytes: <hex_string>
```

#### Example 5: Complex Transaction with Nested Messages

JSON:

```json
{
  "body": {
    "messages": [
      {
        "@type": "/cosmos.bank.v1beta1.MsgSend",
        "from": "cosmos1...abc",
        "to": "cosmos1...def",
        "amount": [
          {
            "denom": "uatom",
            "amount": 10000000
          }
        ]
      },
      {
        "@type": "/cosmos.gov.v1.MsgSubmitProposal",
        "proposer": "cosmos1...ghi",
        "messages": [
          {
            "@type": "/cosmos.bank.v1beta1.MsgSend",
            "from": "cosmos1...jkl",
            "to": "cosmos1...mno",
            "amount": [
              {
                "denom": "uatom",
                "amount": 20000000
              }
            ]
          },
          {
            "@type": "/cosmos.authz.v1beta1.MsgExec",
            "grantee": "cosmos1...pqr",
            "msgs": [
              {
                "@type": "/cosmos.bank.v1beta1.MsgSend",
                "from": "cosmos1...stu",
                "to": "cosmos1...vwx",
                "amount": [
                  {
                    "denom": "uatom",
                    "amount": 30000000
                  }
                ]
              },
              {
                "@type": "/cosmos.bank.v1beta1.MsgSend",
                "from": "cosmos1...abc",
                "to": "cosmos1...def",
                "amount": [
                  {
                    "denom": "uatom",
                    "amount": 40000000
                  }
                ]
              }
            ]
          }
        ],
        "initial_deposit": [
          {
            "denom": "atom",
            "amount": 100.01
          }
        ]
      }
    ]
  },
  "auth_info": {
    "signer_infos": [
      {
        "public_key": "iQ...==",
        "mode_info": { "single": { "mode": "SIGN_MODE_TEXTUAL" } },
        "sequence": 2
      },
      {
        "public_key": "iR...==",
        "mode_info": { "single": { "mode": "SIGN_MODE_DIRECT" } },
        "sequence": 42
      }
    ],
    "fee": {
      "amount": [
        {
          "denom": "atom",
          "amount": 0.002
        }
      ],
      "gas_limit": 100000
    }
  },
  // Additional SignerData.
  "chain_id": "simapp-1",
  "account_number": 10
}
}
```

SIGN_MODE_TEXTUAL for 1st signer `cosmos1...abc`:

```
Chain ID: simapp-1
Account number: 10
*Public Key: iQ...==
Sequence: 2
This transaction has 2 messages:
Message (1/2): bank v1beta1 send coins
From: cosmos1...abc
To: cosmos1...def
Amount: 10 atom
Message (2/2): gov v1 submit proposal
Messages: 2 Messages
> Message (1/2): bank v1beta1 send coins
> From: cosmos1...jkl
> To: cosmos1...mno
> Amount: 20 atom
> Message (2/2): authz v1beta exec
> Grantee: cosmos1...pqr
> Msgs: 2 Msgs
>> Msg (1/2): bank v1beta1 send coins
>> From: cosmos1...stu
>> To: cosmos1...vwx
>> Amount: 30 atom
>> Msg (2/2): bank v1beta1 send coins
>> From: cosmos1...abc
>> To: cosmos1...def
>> Amount: 40 atom
> End of Msgs
End of transaction messages
Proposer: cosmos1...ghi
Initial Deposit: 100.01 atom
End of transaction messages
Fee: 0.002 atom
*Gas: 100'000
*This transaction has 1 other signer:
*Signer (2/2):
*Public Key: iR...==
*Sign mode: Direct
*Sequence: 42
*End of other signers
*Hash of raw bytes: <hex_string>
```

## Consequences

### Backwards Compatibility

SIGN_MODE_TEXTUAL is purely additive, and doesn't break any backwards compatibility with other sign modes.

### Positive

- Human-friendly way of signing in hardware devices.
- Once SIGN_MODE_TEXTUAL is shipped, SIGN_MODE_LEGACY_AMINO_JSON can be deprecated and removed. On the longer term, once the ecosystem has totally migrated, Amino can be totally removed.

### Negative

- Some fields are still encoded in non-human-readable ways, such as public keys in hexadecimal.
- New ledger app needs to be released, still unclear

### Neutral

- If the transaction is complex, the string array can be arbitrarily long, and some users might just skip some screens and blind sign.

## Further Discussions

- Some details on value renderers need to be polished, see [Annex 1](./adr-050-sign-mode-textual-annex1.md).
- Are ledger apps able to support both SIGN_MODE_LEGACY_AMINO_JSON and SIGN_MODE_TEXTUAL at the same time?
- Open question: should we add a Protobuf field option to allow app developers to overwrite the textual representation of certain Protobuf fields and message? This would be similar to Ethereum's [EIP4430](https://github.com/ethereum/EIPs/pull/4430), where the contract developer decides on the textual representation.
- Internationalization.

## References

- [Annex 1](./adr-050-sign-mode-textual-annex1.md)

- Initial discussion: https://github.com/cosmos/cosmos-sdk/issues/6513
- Living document used in the working group: https://hackmd.io/fsZAO-TfT0CKmLDtfMcKeA?both
- Working group meeting notes: https://hackmd.io/7RkGfv_rQAaZzEigUYhcXw
- Ethereum's "Described Transactions" https://github.com/ethereum/EIPs/pull/4430
