- `[privval]` DO NOT require extension signature from privval if vote
  extensions are disabled. Remote signers can skip signing the extension if
  `skip_extension_signing` flag in `SignVoteRequest` is true.
  ([\#2496](https://github.com/cometbft/cometbft/pull/2496))
