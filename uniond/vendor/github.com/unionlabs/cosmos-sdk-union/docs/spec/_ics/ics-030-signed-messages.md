# ICS 030: Cosmos Signed Messages

>TODO: Replace with valid ICS number and possibly move to new location.

* [Changelog](#changelog)
* [Abstract](#abstract)
* [Preliminary](#preliminary)
* [Specification](#specification)
* [Future Adaptations](#future-adaptations)
* [API](#api)
* [References](#references)  

## Status

Proposed.

## Changelog

## Abstract

Having the ability to sign messages off-chain has proven to be a fundamental aspect
of nearly any blockchain. The notion of signing messages off-chain has many
added benefits such as saving on computational costs and reducing transaction
throughput and overhead. Within the context of the Cosmos, some of the major
applications of signing such data includes, but is not limited to, providing a
cryptographic secure and verifiable means of proving validator identity and
possibly associating it with some other framework or organization. In addition,
having the ability to sign Cosmos messages with a Ledger or similar HSM device.

A standardized protocol for hashing, signing, and verifying messages that can be
implemented by the Cosmos SDK and other third-party organizations is needed. Such a
standardized protocol subscribes to the following:

* Contains a specification of human-readable and machine-verifiable typed structured data
* Contains a framework for deterministic and injective encoding of structured data
* Utilizes cryptographic secure hashing and signing algorithms
* A framework for supporting extensions and domain separation
* Is invulnerable to chosen ciphertext attacks
* Has protection against potentially signing transactions a user did not intend to

This specification is only concerned with the rationale and the standardized
implementation of Cosmos signed messages. It does **not** concern itself with the
concept of replay attacks as that will be left up to the higher-level application
implementation. If you view signed messages in the means of authorizing some
action or data, then such an application would have to either treat this as
idempotent or have mechanisms in place to reject known signed messages.

## Preliminary

The Cosmos message signing protocol will be parameterized with a cryptographic
secure hashing algorithm `SHA-256` and a signing algorithm `S` that contains
the operations `sign` and `verify` which provide a digital signature over a set
of bytes and verification of a signature respectively.

Note, our goal here is not to provide context and reasoning about why necessarily
these algorithms were chosen apart from the fact they are the defacto algorithms
used in CometBFT and the Cosmos SDK and that they satisfy our needs for such
cryptographic algorithms such as having resistance to collision and second
pre-image attacks, as well as being [deterministic](https://en.wikipedia.org/wiki/Hash_function#Determinism) and [uniform](https://en.wikipedia.org/wiki/Hash_function#Uniformity).

## Specification

CometBFT has a well established protocol for signing messages using a canonical
JSON representation as defined [here](https://github.com/cometbft/cometbft/blob/master/types/canonical.go).

An example of such a canonical JSON structure is CometBFT's vote structure:

```go
type CanonicalJSONVote struct {
    ChainID   string               `json:"@chain_id"`
    Type      string               `json:"@type"`
    BlockID   CanonicalJSONBlockID `json:"block_id"`
    Height    int64                `json:"height"`
    Round     int                  `json:"round"`
    Timestamp string               `json:"timestamp"`
    VoteType  byte                 `json:"type"`
}
```

With such canonical JSON structures, the specification requires that they include
meta fields: `@chain_id` and `@type`. These meta fields are reserved and must be
included. They are both of type `string`. In addition, fields must be ordered
in lexicographically ascending order.

For the purposes of signing Cosmos messages, the `@chain_id` field must correspond
to the Cosmos chain identifier. The user-agent should **refuse** signing if the
`@chain_id` field does not match the currently active chain! The `@type` field
must equal the constant `"message"`. The `@type` field corresponds to the type of
structure the user will be signing in an application. For now, a user is only
allowed to sign bytes of valid ASCII text ([see here](https://github.com/cometbft/cometbft/blob/v0.38.0/libs/strings/string.go#L57-L67)).
However, this will change and evolve to support additional application-specific
structures that are human-readable and machine-verifiable.

Thus, we can have a canonical JSON structure for signing Cosmos messages using
the [JSON schema](http://json-schema.org/) specification as such:

```json
{
  "$schema": "http://json-schema.org/draft-04/schema#",
  "$id": "cosmos/signing/typeData/schema",
  "title": "The Cosmos signed message typed data schema.",
  "type": "object",
  "properties": {
    "@chain_id": {
      "type": "string",
      "description": "The corresponding Cosmos chain identifier.",
      "minLength": 1
    },
    "@type": {
      "type": "string",
      "description": "The message type. It must be 'message'.",
      "enum": [
        "message"
      ]
    },
    "text": {
      "type": "string",
      "description": "The valid ASCII text to sign.",
      "pattern": "^[\\x20-\\x7E]+$",
      "minLength": 1
    }
  },
  "required": [
    "@chain_id",
    "@type",
    "text"
  ]
}
```

e.g.

```json
{
  "@chain_id": "1",
  "@type": "message",
  "text": "Hello, you can identify me as XYZ on keybase."
}
```

## Future Adaptations

As applications can vary greatly in domain, it will be vital to support both
domain separation and human-readable and machine-verifiable structures.

Domain separation will allow for application developers to prevent collisions of
otherwise identical structures. It should be designed to be unique per application
use and should directly be used in the signature encoding itself.

Human-readable and machine-verifiable structures will allow end users to sign
more complex structures, apart from just string messages, and still be able to
know exactly what they are signing (opposed to signing a bunch of arbitrary bytes).

Thus, in the future, the Cosmos signing message specification will be expected
to expand upon it's canonical JSON structure to include such functionality.

## API

Application developers and designers should formalize a standard set of APIs that
adhere to the following specification:

-----

### **cosmosSignBytes**

Params:

* `data`: the Cosmos signed message canonical JSON structure
* `address`: the Bech32 Cosmos account address to sign data with

Returns:

* `signature`: the Cosmos signature derived using signing algorithm `S`

-----

### Examples

Using the `secp256k1` as the DSA, `S`:

```javascript
data = {
  "@chain_id": "1",
  "@type": "message",
  "text": "I hereby claim I am ABC on Keybase!"
}

cosmosSignBytes(data, "cosmos1pvsch6cddahhrn5e8ekw0us50dpnugwnlfngt3")
> "0x7fc4a495473045022100dec81a9820df0102381cdbf7e8b0f1e2cb64c58e0ecda1324543742e0388e41a02200df37905a6505c1b56a404e23b7473d2c0bc5bcda96771d2dda59df6ed2b98f8"
```

## References
