- `[crypto]` `SupportsBatchVerifier` returns false
  if public key is nil instead of dereferencing nil.
  ([\#1825](https://github.com/cometbft/cometbft/pull/1825))