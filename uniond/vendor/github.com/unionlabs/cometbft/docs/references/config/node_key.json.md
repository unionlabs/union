---
order: 1
parent:
  title: node_key.json
  description: Description and usage of the node ID
  order: 2
---

## The `node_key.json` file
The node ID, the host address and the P2P port together identify a node in a CometBFT network: `nodeID@host:port`.

The easiest way to get the `nodeID` is running the `cometbft show-node-id` command.

The `node_key.json` file resides at `$CMTHOME/config/node_key.json`. This can be overridden at the
[node_key_file](config.toml.md#node_key_file) parameter in the [`config.toml`](config.toml.md) file.

The file contains a private key (in [priv_key.value](#priv_keyvalue)) to an asymmetric algorithm
(in [priv_key.type](#priv_keytype)).

The node ID is calculated by hashing the public key with the SHA256 algorithm and taking the first 20 bytes of the
result.

### priv_key.type
The type of the key defined under [`priv_key.value`](#priv_keyvalue).

Default example in context:
```json
{
  "priv_key": {
    "type": "tendermint/PrivKeyEd25519",
    "value": "jxG2ywUkVPiF4XDW1Dwa5ZfcrC0rEa4iM1y4O5qCMpYxdiypykyf9yp7C81cJTZHKMOvrnGcZiqxlMfyQsaUUA=="
  }
}
```

| Value type          | string (crypto package asymmetric encryption algorithms) |
|:--------------------|:---------------------------------------------------------|
| **Possible values** | `"tendermint/PrivKeyEd25519"`                            |
|                     | `"tendermint/PrivKeySecp256k1"`                          |

The string values are derived from the asymmetric cryptographic implementations defined in the `crypto` package.

CometBFT will always generate an Ed25519 key-pair for node ID using the `cometbft init` or the `cometbft gen-node-key`
commands. Other types of encryption keys have to be created manually. (See examples under
[priv_key.value](#priv_keyvalue).)

### priv_key.value
Base64-encoded bytes, the private key of an asymmetric encryption algorithm.
The type of encryption is defined in [priv_key.type](#priv_keytype).

Default example in context:
```json
{
  "priv_key": {
    "type": "tendermint/PrivKeyEd25519",
    "value": "jxG2ywUkVPiF4XDW1Dwa5ZfcrC0rEa4iM1y4O5qCMpYxdiypykyf9yp7C81cJTZHKMOvrnGcZiqxlMfyQsaUUA=="
  }
}
```

| Value type          | string (base64-encoded bytes)                       |
|:--------------------|:----------------------------------------------------|
| **Possible values** | base64-encoded Ed25519 private key **+ public key** |
|                     | base64-encoded Secp256k1 private key                |

CometBFT will always generate an Ed25519 key-pair for node ID using the `cometbft init` or the `cometbft gen-node-key`
command. Other types of encryption keys have to be created manually. (See examples below.)

The Ed25519 encryption implementation requires the public key concatenated in the value. The implementation ignores the
private key and uses the stored public key to generate the node ID. In the below example, we zeroed out the private key,
but the resultant concatenated bytes still produce a valid node ID. Other algorithms generate the public key from the
private key.

Examples:

Ed25519:
```json
{
  "priv_key": {
    "type": "tendermint/PrivKeyEd25519",
    "value": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAxdiypykyf9yp7C81cJTZHKMOvrnGcZiqxlMfyQsaUUA=="
  }
}
```
Secp256k1:
```json
{
  "priv_key": {
    "type": "tendermint/PrivKeySecp256k1",
    "value": "2swJ5TwUhhqjJW+CvVbbSnTGxqpYmb2yvib+MHyDJIU="
  }
}
```
