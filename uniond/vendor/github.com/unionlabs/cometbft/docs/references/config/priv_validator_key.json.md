---
order: 1
parent:
  title: priv_validator_key.json
  description: Private/public key-pair for signing consensus
  order: 4
---
# priv_validator_key.json
CometBFT supports different key signing methods. The default method is storing the consensus (or signing) key
unencrypted on the file system in the `priv_validator_key.json` file.

The file is located at `$CMTHOME/config/priv_validator_key.json`. If `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft`.

The file contains a [private key](#priv_keyvalue) and its corresponding [public key](#pub_keyvalue).

A [wallet address](#address) is derived from the public key.

### Examples
Ed25519:
```json
{
  "address": "E74FBE24164CFC4F88E311C3AC92E63D0DC310D8",
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "UjxDQgVTlHJOZ7axpMl/iczMIJXiQpFxCFjwKGvzYqE="
  },
  "priv_key": {
    "type": "tendermint/PrivKeyEd25519",
    "value": "9giFjwnmAKCAI95l4Q32kXsau+itGrbsvz84CTLxGnJSPENCBVOUck5ntrGkyX+JzMwgleJCkXEIWPAoa/NioQ=="
  }
}
```

Secp256k1:
```json
{
  "address": "E5B4F106D46A46820308C49B5F92DC22D9F9ACFA",
  "pub_key": {
    "type": "tendermint/PubKeySecp256k1",
    "value": "AhRzbjoZaiyrbCE/yJ6gwIBXjwzl8+H7W8KMAphJVUzt"
  },
  "priv_key": {
    "type": "tendermint/PrivKeySecp256k1",
    "value": "Lfa2uW//4KGvzLXhtoHGfI5Yd2DA2gC7pOfHSkFheGg="
  }
}
```

Do NOT use these examples in production systems unless you are planning to give away your tokens.

You can generate random keys with the `cometbft gen-validator` command.

## address
The wallet address generated from the consensus public key.

The wallet address is calculated by hashing the public key with the SHA256 algorithm and taking the first 20 bytes of
the result.

## pub_key.type
The type of the key defined under [`pub_key.value`](#pub_keyvalue).

| Value type          | string (crypto package asymmetric encryption algorithms) |
|:--------------------|:---------------------------------------------------------|
| **Possible values** | `"tendermint/PubKeyEd25519"`                             |
|                     | `"tendermint/PubKeySecp256k1"`                           |
|                     | `"tendermint/PubKeyBls12_381"`                           |

The string values are derived from the asymmetric cryptographic implementations defined in the `crypto` package.

CometBFT will generate an Ed25519 key-pair for consensus key by default when using the `cometbft init` or the
`cometbft gen-validator` commands. Use `--key-type` or `-k` flag to create a consensus key of a different type.

## pub_key.value
Base64-encoded bytes, the public key of an asymmetric encryption algorithm.
The type of encryption is defined in [pub_key.type](#pub_keytype).

| Value type          | string (base64-encoded bytes)       |
|:--------------------|:------------------------------------|
| **Possible values** | base64-encoded Ed25519 public key   |
|                     | base64-encoded Secp256k1 public key |
|                     | base64-encoded BLS12-381 public key |

CometBFT will generate an Ed25519 key-pair for consensus key by default when using the `cometbft init` or the
`cometbft gen-validator` commands. Use `--key-type` or `-k` flag to create a consensus key of a different type.


## priv_key.type
The type of the key defined under [`priv_key.value`](#priv_keyvalue).

| Value type          | string (crypto package asymmetric encryption algorithms) |
|:--------------------|:---------------------------------------------------------|
| **Possible values** | `"tendermint/PrivKeyEd25519"`                            |
|                     | `"tendermint/PrivKeySecp256k1"`                          |
|                     | `"tendermint/PrivKeyBls12_381"`                          |

The string values are derived from the asymmetric cryptographic implementations defined in the `crypto` package.

CometBFT will generate an Ed25519 key-pair for consensus key by default when using the `cometbft init` or the
`cometbft gen-validator` commands. Use `--key-type` or `-k` flag to create a consensus key of a different type.

## priv_key.value
Base64-encoded bytes, the private key of an asymmetric encryption algorithm.
The type of encryption is defined in [priv_key.type](#priv_keytype).

| Value type          | string (base64-encoded bytes)                       |
|:--------------------|:----------------------------------------------------|
| **Possible values** | base64-encoded Ed25519 private key **+ public key** |
|                     | base64-encoded Secp256k1 private key                |
|                     | base64-encoded BLS12-381 private key                |

CometBFT will generate an Ed25519 key-pair for consensus key by default when using the `cometbft init` or the
`cometbft gen-validator` commands. Use `--key-type` or `-k` flag to create a consensus key of a different type.

The Ed25519 encryption implementation requires the public key concatenated in the value.
