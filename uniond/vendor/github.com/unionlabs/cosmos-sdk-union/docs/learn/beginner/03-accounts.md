---
sidebar_position: 1
---

# Accounts

:::note Synopsis
This document describes the in-built account and public key system of the Cosmos SDK.
:::

:::note Pre-requisite Readings


* [Anatomy of a Cosmos SDK Application](./00-app-anatomy.md)

:::

## Account Definition

In the Cosmos SDK, an _account_ designates a pair of _public key_ `PubKey` and _private key_ `PrivKey`. The `PubKey` can be derived to generate various `Addresses`, which are used to identify users (among other parties) in the application. `Addresses` are also associated with [`message`s](../../build/building-modules/02-messages-and-queries.md#messages) to identify the sender of the `message`. The `PrivKey` is used to generate [digital signatures](#keys-accounts-addresses-and-signatures) to prove that an `Address` associated with the `PrivKey` approved of a given `message`.

For HD key derivation the Cosmos SDK uses a standard called [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki). The BIP32 allows users to create an HD wallet (as specified in [BIP44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)) - a set of accounts derived from an initial secret seed. A seed is usually created from a 12- or 24-word mnemonic. A single seed can derive any number of `PrivKey`s using a one-way cryptographic function. Then, a `PubKey` can be derived from the `PrivKey`. Naturally, the mnemonic is the most sensitive information, as private keys can always be re-generated if the mnemonic is preserved.

```mermaid
graph BT
    A0A[Address 0] --> A0[Account 0]
    A0PK[Public key 0] --> A0A[Address 0]
    A0SK[Private key 0] --> A0PK[Public key 0]
    A1A[Address 1] --> A1[Account 1]
    A1PK[Public key 1] --> A1A[Address 1]
    A1SK[Private key 1] --> A1PK[Public key 1]
    A2A[Address 2] --> A2[Account 2]
    A2PK[Public key 2] --> A2A[Address 2]
    A2SK[Private key 2] --> A2PK[Public key 2]
    MasterPK[Master PrivKey] --> A0SK[Private key 0]
    MasterPK[Master PrivKey] --> A1SK[Private key 1]
    MasterPK[Master PrivKey] --> A2SK[Private key 2]
    Mnemonic["Mnemonic (Seed)"] --> MasterPK[Master PrivKey]
```

In the Cosmos SDK, keys are stored and managed by using an object called a [`Keyring`](#keyring).

## Keys, accounts, addresses, and signatures

The principal way of authenticating a user is done using [digital signatures](https://en.wikipedia.org/wiki/Digital_signature). Users sign transactions using their own private key. Signature verification is done with the associated public key. For on-chain signature verification purposes, we store the public key in an `Account` object (alongside other data required for a proper transaction validation).

In the node, all data is stored using Protocol Buffers serialization.

The Cosmos SDK supports the following digital key schemes for creating digital signatures:

* `secp256k1`, as implemented in the [Cosmos SDK's `crypto/keys/secp256k1` package](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keys/secp256k1/secp256k1.go).
* `secp256r1`, as implemented in the [Cosmos SDK's `crypto/keys/secp256r1` package](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keys/secp256r1/pubkey.go),
* `tm-ed25519`, as implemented in the [Cosmos SDK `crypto/keys/ed25519` package](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keys/ed25519/ed25519.go). This scheme is supported only for the consensus validation.

|              | Address length in bytes | Public key length in bytes | Used for transaction authentication | Used for consensus (cometbft) |
| :----------: | :---------------------: | :------------------------: | :---------------------------------: | :-----------------------------: |
| `secp256k1`  |           20            |             33             |                 yes                 |               no                |
| `secp256r1`  |           32            |             33             |                 yes                 |               no                |
| `tm-ed25519` |     -- not used --      |             32             |                 no                  |               yes               |

## Addresses

`Addresses` and `PubKey`s are both public information that identifies actors in the application. `Account` is used to store authentication information. The basic account implementation is provided by a `BaseAccount` object.

Each account is identified using `Address` which is a sequence of bytes derived from a public key. In the Cosmos SDK, we define 3 types of addresses that specify a context where an account is used:

* `AccAddress` identifies users (the sender of a `message`).
* `ValAddress` identifies validator operators.
* `ConsAddress` identifies validator nodes that are participating in consensus. Validator nodes are derived using the **`ed25519`** curve.

These types implement the `Address` interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/types/address.go#L126-L134
```

Address construction algorithm is defined in [ADR-28](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-028-public-key-addresses.md).
Here is the standard way to obtain an account address from a `pub` public key:

```go
sdk.AccAddress(pub.Address().Bytes())
```

Of note, the `Marshal()` and `Bytes()` method both return the same raw `[]byte` form of the address. `Marshal()` is required for Protobuf compatibility.

For user interaction, addresses are formatted using [Bech32](https://en.bitcoin.it/wiki/Bech32) and implemented by the `String` method. The Bech32 method is the only supported format to use when interacting with a blockchain. The Bech32 human-readable part (Bech32 prefix) is used to denote an address type. Example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/types/address.go#L299-L316
```

|                    | Address Bech32 Prefix |
| ------------------ | --------------------- |
| Accounts           | cosmos                |
| Validator Operator | cosmosvaloper         |
| Consensus Nodes    | cosmosvalcons         |

### Public Keys

Public keys in Cosmos SDK are defined by `cryptotypes.PubKey` interface. Since public keys are saved in a store, `cryptotypes.PubKey` extends the `proto.Message` interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/types/types.go#L8-L17
```

A compressed format is used for `secp256k1` and `secp256r1` serialization.

* The first byte is a `0x02` byte if the `y`-coordinate is the lexicographically largest of the two associated with the `x`-coordinate.
* Otherwise the first byte is a `0x03`.

This prefix is followed by the `x`-coordinate.

Public Keys are not used to reference accounts (or users) and in general are not used when composing transaction messages (with few exceptions: `MsgCreateValidator`, `Validator` and `Multisig` messages).
For user interactions, `PubKey` is formatted using Protobufs JSON ([ProtoMarshalJSON](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/codec/json.go#L14-L34) function). Example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/client/keys/output.go#L23-L39
```

## Keyring

A `Keyring` is an object that stores and manages accounts. In the Cosmos SDK, a `Keyring` implementation follows the `Keyring` interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keyring/keyring.go#L57-L105
```

The default implementation of `Keyring` comes from the third-party [`99designs/keyring`](https://github.com/99designs/keyring) library.

A few notes on the `Keyring` methods:

* `Sign(uid string, msg []byte) ([]byte, types.PubKey, error)` strictly deals with the signature of the `msg` bytes. You must prepare and encode the transaction into a canonical `[]byte` form. Because protobuf is not deterministic, it has been decided in [ADR-020](../../architecture/adr-020-protobuf-transaction-encoding.md) that the canonical `payload` to sign is the `SignDoc` struct, deterministically encoded using [ADR-027](../../architecture/adr-027-deterministic-protobuf-serialization.md). Note that signature verification is not implemented in the Cosmos SDK by default, it is deferred to the [`anteHandler`](../advanced/00-baseapp.md#antehandler).

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/proto/cosmos/tx/v1beta1/tx.proto#L50-L66
```

* `NewAccount(uid, mnemonic, bip39Passphrase, hdPath string, algo SignatureAlgo) (*Record, error)` creates a new account based on the [`bip44 path`](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki) and persists it on disk. The `PrivKey` is **never stored unencrypted**, instead it is [encrypted with a passphrase](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/armor.go) before being persisted. In the context of this method, the key type and sequence number refers to the segment of the BIP44 derivation path (for example, `0`, `1`, `2`, ...) that is used to derive a private and a public key from the mnemonic. Using the same mnemonic and derivation path, the same `PrivKey`, `PubKey` and `Address` is generated. The following keys are supported by the keyring:

* `secp256k1`
* `ed25519`

* `ExportPrivKeyArmor(uid, encryptPassphrase string) (armor string, err error)` exports a private key in ASCII-armored encrypted format using the given passphrase. You can then either import the private key again into the keyring using the `ImportPrivKey(uid, armor, passphrase string)` function or decrypt it into a raw private key using the `UnarmorDecryptPrivKey(armorStr string, passphrase string)` function.

### Create New Key Type

To create a new key type for using in keyring, `keyring.SignatureAlgo` interface must be fulfilled.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keyring/signing_algorithms.go#L10-L15
```

The interface consists in three methods where `Name()` returns the name of the algorithm as a `hd.PubKeyType` and `Derive()` and `Generate()` must return the following functions respectively:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/hd/algo.go#L28-L31
```

Once the `keyring.SignatureAlgo` has been implemented it must be added to the [list of supported algos](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keyring/keyring.go#L217) of the keyring.

For simplicity the implementation of a new key type should be done inside the `crypto/hd` package.
There is an example of a working `secp256k1` implementation in [algo.go](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/hd/algo.go#L38).


#### Implementing secp256r1 algo

Here is an example of how secp256r1 could be implemented.

First a new function to create a private key from a secret number is needed in the secp256r1 package. This function could look like this:

```go
// cosmos-sdk/crypto/keys/secp256r1/privkey.go

// NewPrivKeyFromSecret creates a private key derived for the secret number
// represented in big-endian. The `secret` must be a valid ECDSA field element.
func NewPrivKeyFromSecret(secret []byte) (*PrivKey, error) {
	var d = new(big.Int).SetBytes(secret)
	if d.Cmp(secp256r1.Params().N) >= 1 {
		return nil, errorsmod.Wrap(errors.ErrInvalidRequest, "secret not in the curve base field")
	}
	sk := new(ecdsa.PrivKey)
	return &PrivKey{&ecdsaSK{*sk}}, nil
}
```

After that `secp256r1Algo` can be implemented.

```go
// cosmos-sdk/crypto/hd/secp256r1Algo.go

package hd

import (
	"github.com/cosmos/go-bip39"
	
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256r1"
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

// Secp256r1Type uses the secp256r1 ECDSA parameters.
const Secp256r1Type = PubKeyType("secp256r1")

var Secp256r1 = secp256r1Algo{}

type secp256r1Algo struct{}

func (s secp256r1Algo) Name() PubKeyType {
	return Secp256r1Type
}

// Derive derives and returns the secp256r1 private key for the given seed and HD path.
func (s secp256r1Algo) Derive() DeriveFn {
	return func(mnemonic string, bip39Passphrase, hdPath string) ([]byte, error) {
		seed, err := bip39.NewSeedWithErrorChecking(mnemonic, bip39Passphrase)
		if err != nil {
			return nil, err
		}

		masterPriv, ch := ComputeMastersFromSeed(seed)
		if len(hdPath) == 0 {
			return masterPriv[:], nil
		}
		derivedKey, err := DerivePrivateKeyForPath(masterPriv, ch, hdPath)

		return derivedKey, err
	}
}

// Generate generates a secp256r1 private key from the given bytes.
func (s secp256r1Algo) Generate() GenerateFn {
	return func(bz []byte) types.PrivKey {
		key, err := secp256r1.NewPrivKeyFromSecret(bz)
		if err != nil {
			panic(err)
		}
		return key
	}
}
```

Finally, the algo must be added to the list of [supported algos](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/crypto/keyring/keyring.go#L217) by the keyring.

```go
// cosmos-sdk/crypto/keyring/keyring.go

func newKeystore(kr keyring.Keyring, cdc codec.Codec, backend string, opts ...Option) keystore {
	// Default options for keybase, these can be overwritten using the
	// Option function
	options := Options{
		SupportedAlgos:       SigningAlgoList{hd.Secp256k1, hd.Secp256r1}, // added here
		SupportedAlgosLedger: SigningAlgoList{hd.Secp256k1},
	}
...
```

Hereafter to create new keys using your algo, you must specify it with the flag `--algo` :

`simd keys add myKey --algo secp256r1`
