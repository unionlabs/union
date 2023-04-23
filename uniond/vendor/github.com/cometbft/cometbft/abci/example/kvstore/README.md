# KVStore

There are two app's here: the KVStoreApplication and the PersistentKVStoreApplication.

## KVStoreApplication

The KVStoreApplication is a simple merkle key-value store. 
Transactions of the form `key=value` are stored as key-value pairs in the tree.
Transactions without an `=` sign set the value to the key.
The app has no replay protection (other than what the mempool provides).

## PersistentKVStoreApplication

The PersistentKVStoreApplication wraps the KVStoreApplication
and provides two additional features:

1) persistence of state across app restarts (using CometBFT's ABCI-Handshake mechanism)
2) validator set changes

The state is persisted in leveldb along with the last block committed,
and the Handshake allows any necessary blocks to be replayed.
Validator set changes are effected using the following transaction format:

```md
"val:pubkey1!power1,pubkey2!power2,pubkey3!power3"
```

where `pubkeyN` is a base64-encoded 32-byte ed25519 key and `powerN` is a new voting power for the validator with `pubkeyN` (possibly a new one).
To remove a validator from the validator set, set power to `0`.
There is no sybil protection against new validators joining. 
