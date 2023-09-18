# ADR 020: Protocol Buffer Transaction Encoding

## Changelog

* 2020 March 06: Initial Draft
* 2020 March 12: API Updates
* 2020 April 13: Added details on interface `oneof` handling
* 2020 April 30: Switch to `Any`
* 2020 May 14: Describe public key encoding
* 2020 June 08: Store `TxBody` and `AuthInfo` as bytes in `SignDoc`; Document `TxRaw` as broadcast and storage type.
* 2020 August 07: Use ADR 027 for serializing `SignDoc`.
* 2020 August 19: Move sequence field from `SignDoc` to `SignerInfo`, as discussed in [#6966](https://github.com/cosmos/cosmos-sdk/issues/6966).
* 2020 September 25: Remove `PublicKey` type in favor of `secp256k1.PubKey`, `ed25519.PubKey` and `multisig.LegacyAminoPubKey`.
* 2020 October 15: Add `GetAccount` and `GetAccountWithHeight` methods to the `AccountRetriever` interface.
* 2021 Feb 24: The Cosmos SDK does not use Tendermint's `PubKey` interface anymore, but its own `cryptotypes.PubKey`. Updates to reflect this.
* 2021 May 3: Rename `clientCtx.JSONMarshaler` to `clientCtx.JSONCodec`.
* 2021 June 10: Add `clientCtx.Codec: codec.Codec`.

## Status

Accepted

## Context

This ADR is a continuation of the motivation, design, and context established in
[ADR 019](./adr-019-protobuf-state-encoding.md), namely, we aim to design the
Protocol Buffer migration path for the client-side of the Cosmos SDK.

Specifically, the client-side migration path primarily includes tx generation and
signing, message construction and routing, in addition to CLI & REST handlers and
business logic (i.e. queriers).

With this in mind, we will tackle the migration path via two main areas, txs and
querying. However, this ADR solely focuses on transactions. Querying should be
addressed in a future ADR, but it should build off of these proposals.

Based on detailed discussions ([\#6030](https://github.com/cosmos/cosmos-sdk/issues/6030)
and [\#6078](https://github.com/cosmos/cosmos-sdk/issues/6078)), the original
design for transactions was changed substantially from an `oneof` /JSON-signing
approach to the approach described below.

## Decision

### Transactions

Since interface values are encoded with `google.protobuf.Any` in state (see [ADR 019](adr-019-protobuf-state-encoding.md)),
`sdk.Msg`s are encoding with `Any` in transactions.

One of the main goals of using `Any` to encode interface values is to have a
core set of types which is reused by apps so that
clients can safely be compatible with as many chains as possible.

It is one of the goals of this specification to provide a flexible cross-chain transaction
format that can serve a wide variety of use cases without breaking client
compatibility.

In order to facilitate signing, transactions are separated into `TxBody`,
which will be re-used by `SignDoc` below, and `signatures`:

```protobuf
// types/types.proto
package cosmos_sdk.v1;

message Tx {
    TxBody body = 1;
    AuthInfo auth_info = 2;
    // A list of signatures that matches the length and order of AuthInfo's signer_infos to
    // allow connecting signature meta information like public key and signing mode by position.
    repeated bytes signatures = 3;
}

// A variant of Tx that pins the signer's exact binary represenation of body and
// auth_info. This is used for signing, broadcasting and verification. The binary
// `serialize(tx: TxRaw)` is stored in Tendermint and the hash `sha256(serialize(tx: TxRaw))`
// becomes the "txhash", commonly used as the transaction ID.
message TxRaw {
    // A protobuf serialization of a TxBody that matches the representation in SignDoc.
    bytes body = 1;
    // A protobuf serialization of an AuthInfo that matches the representation in SignDoc.
    bytes auth_info = 2;
    // A list of signatures that matches the length and order of AuthInfo's signer_infos to
    // allow connecting signature meta information like public key and signing mode by position.
    repeated bytes signatures = 3;
}

message TxBody {
    // A list of messages to be executed. The required signers of those messages define
    // the number and order of elements in AuthInfo's signer_infos and Tx's signatures.
    // Each required signer address is added to the list only the first time it occurs.
    //
    // By convention, the first required signer (usually from the first message) is referred
    // to as the primary signer and pays the fee for the whole transaction.
    repeated google.protobuf.Any messages = 1;
    string memo = 2;
    int64 timeout_height = 3;
    repeated google.protobuf.Any extension_options = 1023;
}

message AuthInfo {
    // This list defines the signing modes for the required signers. The number
    // and order of elements must match the required signers from TxBody's messages.
    // The first element is the primary signer and the one which pays the fee.
    repeated SignerInfo signer_infos = 1;
    // The fee can be calculated based on the cost of evaluating the body and doing signature verification of the signers. This can be estimated via simulation.
    Fee fee = 2;
}

message SignerInfo {
    // The public key is optional for accounts that already exist in state. If unset, the
    // verifier can use the required signer address for this position and lookup the public key.
    google.protobuf.Any public_key = 1;
    // ModeInfo describes the signing mode of the signer and is a nested
    // structure to support nested multisig pubkey's
    ModeInfo mode_info = 2;
    // sequence is the sequence of the account, which describes the
    // number of committed transactions signed by a given address. It is used to prevent
    // replay attacks.
    uint64 sequence = 3;
}

message ModeInfo {
    oneof sum {
        Single single = 1;
        Multi multi = 2;
    }

    // Single is the mode info for a single signer. It is structured as a message
    // to allow for additional fields such as locale for SIGN_MODE_TEXTUAL in the future
    message Single {
        SignMode mode = 1;
    }

    // Multi is the mode info for a multisig public key
    message Multi {
        // bitarray specifies which keys within the multisig are signing
        CompactBitArray bitarray = 1;
        // mode_infos is the corresponding modes of the signers of the multisig
        // which could include nested multisig public keys
        repeated ModeInfo mode_infos = 2;
    }
}

enum SignMode {
    SIGN_MODE_UNSPECIFIED = 0;

    SIGN_MODE_DIRECT = 1;

    SIGN_MODE_TEXTUAL = 2;

    SIGN_MODE_LEGACY_AMINO_JSON = 127;
}
```

As will be discussed below, in order to include as much of the `Tx` as possible
in the `SignDoc`, `SignerInfo` is separated from signatures so that only the
raw signatures themselves live outside of what is signed over.

Because we are aiming for a flexible, extensible cross-chain transaction
format, new transaction processing options should be added to `TxBody` as soon
those use cases are discovered, even if they can't be implemented yet.

Because there is coordination overhead in this, `TxBody` includes an
`extension_options` field which can be used for any transaction processing
options that are not already covered. App developers should, nevertheless,
attempt to upstream important improvements to `Tx`.

### Signing

All of the signing modes below aim to provide the following guarantees:

* **No Malleability**: `TxBody` and `AuthInfo` cannot change once the transaction
  is signed
* **Predictable Gas**: if I am signing a transaction where I am paying a fee,
  the final gas is fully dependent on what I am signing

These guarantees give the maximum amount confidence to message signers that
manipulation of `Tx`s by intermediaries can't result in any meaningful changes.

#### `SIGN_MODE_DIRECT`

The "direct" signing behavior is to sign the raw `TxBody` bytes as broadcast over
the wire. This has the advantages of:

* requiring the minimum additional client capabilities beyond a standard protocol
  buffers implementation
* leaving effectively zero holes for transaction malleability (i.e. there are no
  subtle differences between the signing and encoding formats which could
  potentially be exploited by an attacker)

Signatures are structured using the `SignDoc` below which reuses the serialization of
`TxBody` and `AuthInfo` and only adds the fields which are needed for signatures:

```protobuf
// types/types.proto
message SignDoc {
    // A protobuf serialization of a TxBody that matches the representation in TxRaw.
    bytes body = 1;
    // A protobuf serialization of an AuthInfo that matches the representation in TxRaw.
    bytes auth_info = 2;
    string chain_id = 3;
    uint64 account_number = 4;
}
```

In order to sign in the default mode, clients take the following steps:

1. Serialize `TxBody` and `AuthInfo` using any valid protobuf implementation.
2. Create a `SignDoc` and serialize it using [ADR 027](./adr-027-deterministic-protobuf-serialization.md).
3. Sign the encoded `SignDoc` bytes.
4. Build a `TxRaw` and serialize it for broadcasting.

Signature verification is based on comparing the raw `TxBody` and `AuthInfo`
bytes encoded in `TxRaw` not based on any ["canonicalization"](https://github.com/regen-network/canonical-proto3)
algorithm which creates added complexity for clients in addition to preventing
some forms of upgradeability (to be addressed later in this document).

Signature verifiers do:

1. Deserialize a `TxRaw` and pull out `body` and `auth_info`.
2. Create a list of required signer addresses from the messages.
3. For each required signer:
   * Pull account number and sequence from the state.
   * Obtain the public key either from state or `AuthInfo`'s `signer_infos`.
   * Create a `SignDoc` and serialize it using [ADR 027](./adr-027-deterministic-protobuf-serialization.md).
   * Verify the signature at the same list position against the serialized `SignDoc`.

#### `SIGN_MODE_LEGACY_AMINO`

In order to support legacy wallets and exchanges, Amino JSON will be temporarily
supported transaction signing. Once wallets and exchanges have had a
chance to upgrade to protobuf based signing, this option will be disabled. In
the meantime, it is foreseen that disabling the current Amino signing would cause
too much breakage to be feasible. Note that this is mainly a requirement of the
Cosmos Hub and other chains may choose to disable Amino signing immediately.

Legacy clients will be able to sign a transaction using the current Amino
JSON format and have it encoded to protobuf using the REST `/tx/encode`
endpoint before broadcasting.

#### `SIGN_MODE_TEXTUAL`

As was discussed extensively in [\#6078](https://github.com/cosmos/cosmos-sdk/issues/6078),
there is a desire for a human-readable signing encoding, especially for hardware
wallets like the [Ledger](https://www.ledger.com) which display
transaction contents to users before signing. JSON was an attempt at this but
falls short of the ideal.

`SIGN_MODE_TEXTUAL` is intended as a placeholder for a human-readable
encoding which will replace Amino JSON. This new encoding should be even more
focused on readability than JSON, possibly based on formatting strings like
[MessageFormat](http://userguide.icu-project.org/formatparse/messages).

In order to ensure that the new human-readable format does not suffer from
transaction malleability issues, `SIGN_MODE_TEXTUAL`
requires that the _human-readable bytes are concatenated with the raw `SignDoc`_
to generate sign bytes.

Multiple human-readable formats (maybe even localized messages) may be supported
by `SIGN_MODE_TEXTUAL` when it is implemented.

### Unknown Field Filtering

Unknown fields in protobuf messages should generally be rejected by transaction
processors because:

* important data may be present in the unknown fields, that if ignored, will
  cause unexpected behavior for clients
* they present a malleability vulnerability where attackers can bloat tx size
  by adding random uninterpreted data to unsigned content (i.e. the master `Tx`,
  not `TxBody`)

There are also scenarios where we may choose to safely ignore unknown fields
(https://github.com/cosmos/cosmos-sdk/issues/6078#issuecomment-624400188) to
provide graceful forwards compatibility with newer clients.

We propose that field numbers with bit 11 set (for most use cases this is
the range of 1024-2047) be considered non-critical fields that can safely be
ignored if unknown.

To handle this we will need a unknown field filter that:

* always rejects unknown fields in unsigned content (i.e. top-level `Tx` and
  unsigned parts of `AuthInfo` if present based on the signing mode)
* rejects unknown fields in all messages (including nested `Any`s) other than
  fields with bit 11 set

This will likely need to be a custom protobuf parser pass that takes message bytes
and `FileDescriptor`s and returns a boolean result.

### Public Key Encoding

Public keys in the Cosmos SDK implement the `cryptotypes.PubKey` interface.
We propose to use `Any` for protobuf encoding as we are doing with other interfaces (for example, in `BaseAccount.PubKey` and `SignerInfo.PublicKey`).
The following public keys are implemented: secp256k1, secp256r1, ed25519 and legacy-multisignature.

Ex:

```protobuf
message PubKey {
    bytes key = 1;
}
```

`multisig.LegacyAminoPubKey` has an array of `Any`'s member to support any
protobuf public key type.

Apps should only attempt to handle a registered set of public keys that they
have tested. The provided signature verification ante handler decorators will
enforce this.

### CLI & REST

Currently, the REST and CLI handlers encode and decode types and txs via Amino
JSON encoding using a concrete Amino codec. Being that some of the types dealt with
in the client can be interfaces, similar to how we described in [ADR 019](./adr-019-protobuf-state-encoding.md),
the client logic will now need to take a codec interface that knows not only how
to handle all the types, but also knows how to generate transactions, signatures,
and messages.

```go
type AccountRetriever interface {
  GetAccount(clientCtx Context, addr sdk.AccAddress) (client.Account, error)
  GetAccountWithHeight(clientCtx Context, addr sdk.AccAddress) (client.Account, int64, error)
  EnsureExists(clientCtx client.Context, addr sdk.AccAddress) error
  GetAccountNumberSequence(clientCtx client.Context, addr sdk.AccAddress) (uint64, uint64, error)
}

type Generator interface {
  NewTx() TxBuilder
  NewFee() ClientFee
  NewSignature() ClientSignature
  MarshalTx(tx types.Tx) ([]byte, error)
}

type TxBuilder interface {
  GetTx() sdk.Tx

  SetMsgs(...sdk.Msg) error
  GetSignatures() []sdk.Signature
  SetSignatures(...sdk.Signature)
  GetFee() sdk.Fee
  SetFee(sdk.Fee)
  GetMemo() string
  SetMemo(string)
}
```

We then update `Context` to have new fields: `Codec`, `TxGenerator`,
and `AccountRetriever`, and we update `AppModuleBasic.GetTxCmd` to take
a `Context` which should have all of these fields pre-populated.

Each client method should then use one of the `Init` methods to re-initialize
the pre-populated `Context`. `tx.GenerateOrBroadcastTx` can be used to
generate or broadcast a transaction. For example:

```go
import "github.com/spf13/cobra"
import "github.com/cosmos/cosmos-sdk/client"
import "github.com/cosmos/cosmos-sdk/client/tx"

func NewCmdDoSomething(clientCtx client.Context) *cobra.Command {
	return &cobra.Command{
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := ctx.InitWithInput(cmd.InOrStdin())
			msg := NewSomeMsg{...}
			tx.GenerateOrBroadcastTx(clientCtx, msg)
		},
	}
}
```

## Future Improvements

### `SIGN_MODE_TEXTUAL` specification

A concrete specification and implementation of `SIGN_MODE_TEXTUAL` is intended
as a near-term future improvement so that the ledger app and other wallets
can gracefully transition away from Amino JSON.

### `SIGN_MODE_DIRECT_AUX`

(\*Documented as option (3) in https://github.com/cosmos/cosmos-sdk/issues/6078#issuecomment-628026933)

We could add a mode `SIGN_MODE_DIRECT_AUX`
to support scenarios where multiple signatures
are being gathered into a single transaction but the message composer does not
yet know which signatures will be included in the final transaction. For instance,
I may have a 3/5 multisig wallet and want to send a `TxBody` to all 5
signers to see who signs first. As soon as I have 3 signatures then I will go
ahead and build the full transaction.

With `SIGN_MODE_DIRECT`, each signer needs
to sign the full `AuthInfo` which includes the full list of all signers and
their signing modes, making the above scenario very hard.

`SIGN_MODE_DIRECT_AUX` would allow "auxiliary" signers to create their signature
using only `TxBody` and their own `PublicKey`. This allows the full list of
signers in `AuthInfo` to be delayed until signatures have been collected.

An "auxiliary" signer is any signer besides the primary signer who is paying
the fee. For the primary signer, the full `AuthInfo` is actually needed to calculate gas and fees
because that is dependent on how many signers and which key types and signing
modes they are using. Auxiliary signers, however, do not need to worry about
fees or gas and thus can just sign `TxBody`.

To generate a signature in `SIGN_MODE_DIRECT_AUX` these steps would be followed:

1. Encode `SignDocAux` (with the same requirement that fields must be serialized
   in order):

    ```protobuf
    // types/types.proto
    message SignDocAux {
        bytes body_bytes = 1;
        // PublicKey is included in SignDocAux :
        // 1. as a special case for multisig public keys. For multisig public keys,
        // the signer should use the top-level multisig public key they are signing
        // against, not their own public key. This is to prevent against a form
        // of malleability where a signature could be taken out of context of the
        // multisig key that was intended to be signed for
        // 2. to guard against scenario where configuration information is encoded
        // in public keys (it has been proposed) such that two keys can generate
        // the same signature but have different security properties
        //
        // By including it here, the composer of AuthInfo cannot reference the
        // a public key variant the signer did not intend to use
        PublicKey public_key = 2;
        string chain_id = 3;
        uint64 account_number = 4;
    }
    ```

2. Sign the encoded `SignDocAux` bytes
3. Send their signature and `SignerInfo` to primary signer who will then
   sign and broadcast the final transaction (with `SIGN_MODE_DIRECT` and `AuthInfo`
   added) once enough signatures have been collected

### `SIGN_MODE_DIRECT_RELAXED`

(_Documented as option (1)(a) in https://github.com/cosmos/cosmos-sdk/issues/6078#issuecomment-628026933_)

This is a variation of `SIGN_MODE_DIRECT` where multiple signers wouldn't need to
coordinate public keys and signing modes in advance. It would involve an alternate
`SignDoc` similar to `SignDocAux` above with fee. This could be added in the future
if client developers found the burden of collecting public keys and modes in advance
too burdensome.

## Consequences

### Positive

* Significant performance gains.
* Supports backward and forward type compatibility.
* Better support for cross-language clients.
* Multiple signing modes allow for greater protocol evolution

### Negative

* `google.protobuf.Any` type URLs increase transaction size although the effect
  may be negligible or compression may be able to mitigate it.

### Neutral

## References
