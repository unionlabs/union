# Attested Light Client

This contract provides a trusted, attestation-based light client for chains that have no verifiable consensus or state.

## Configuration

This contract supports attestations for multiple chains. For each chain that will be attested to, a quorum and the attestors must be configured.

To configure the quorum:

```jsonc
{
  "set_quorum": {
    "chain_id": "...",
    "quorum": /* non-zero u8 */
  }
}
```

To add an attestor:

```jsonc
{
  "add_attestor": {
    "chain_id": "...",
    "new_attestor": /* 0x-prefixed attestor pubkey (ed25519) */
  }
}
```

Attestors can also be removed with a similar message:

```jsonc
{
  "remove_attestor": {
    "chain_id": "...",
    "old_attestor": /* 0x-prefixed attestor pubkey (ed25519) */
  }
}
```

Note that by default, these messages can only be executed by a configured admin (see ../../../access-managed for more information on access management).

## Attestation Flow

Attestations are submitted with the `attest` message:

```jsonc
{
  "attest": {
    "attestation": {
      "chain_id": "...",
      "height": /* u64 height */,
      "timestamp": /* u64 timestamp, nanoseconds */,
      "key": /* 0x-prefixed key bytes */,
      "value": /* attested value */
    },
    "attestor": /* 0x-prefixed attestor pubkey (ed25519) */,
    "signature": /* 0x-prefixed attestor signature (ed25519) */
  }
}
```

The attested value can either be existence or non-existence.

For existence:

```jsonc
{
  "existence": /* 0x-prefixed value bytes */
}
```

For non-existence:

```jsonc
"non_existence"
```

Note that all attestations contain a height and a timestamp. Rather than having a second attestation flow for height timestamps (which are required by the union IBC light client specification), the timestamps for a height are derived from the attestations for that height. Given this, once an attestation exists at a height, all future attestations at that same height must contain the same timestamp.

### Attestor Service

An existing attestor service for EVM-compatible chains is provided via a [Voyager plugin][attestor-evm]. See the plugin's documentation for how to run the attestor service. Alternative implementations are welcome, both for EVM-compatible chains and other execution environments.

## Light Client Flow

This contract fully supports the union IBC light client specification, as follows:

### `get_timestamp`

Returns the current `consensus_state.timestamp`.

### `get_latest_height`

Returns the current `client_state.latest_height`.

### `status`

Always returns `active`.

### `verify_creation`

Always an empty success response.

### `verify_membership`

Verifies the provided key:value pair at the provided height by verifying that an attestation for the (chain id, height, key) tuple exists, and that the attested is value equal.

### `verify_non_membership`

Verifies the provided key at the provided height by verifying that an attestation for the (chain id, height, key) tuple exists, and that the attested value is non-existence.

### `verify_header`

This is largely just a stub implementation, and no actual verifications are performed, other than ensuring that the provided height is attested to be the provided timestamp. Both of these values can be queried directly from the contract, so anyone can permissionlessly update the client once an attestation has been confirmed. This functionality enables this client to be transparently supported by the union IBC stack.

### `misbehaviour`

Misbehaviour is not supported.

## Additional Queries

In addition to the standard light client interface, this contract also exposes the following queries.

Query the quorum for a configured chain:

```jsonc
{
  "quorum": {
    "chain_id": "..."
  }
}
```

Query the configured attestors for a chain:

```jsonc
{
  "attestors": {
    "chain_id": "..."
  }
}
```

Query the value attested to under a key at a height for a chain:

```jsonc
{
  "attestors": {
    "chain_id": "...",
    "height": /* u64 height */,
    "key": /* 0x-prefixed key bytes */
  }
}
```

Query the timestamp attested to at a height for a chain:

```jsonc
{
  "timestamp_at_height": {
    "chain_id": "...",
    "height": /* u64 height */
  }
}
```

Query the latest attested height for a chain:

```jsonc
{
  "latest_height": {
    "chain_id": "...",
  }
}
```

[attestor-evm]: ../../../../voyager/plugins/attestor/evm
