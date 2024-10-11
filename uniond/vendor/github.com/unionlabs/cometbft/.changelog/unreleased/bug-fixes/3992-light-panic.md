- `[light]` Return and log an error when starting from an empty trusted store.
  This can happen using the `light` CometBFT command-line command while using
  a fresh trusted store and no trusted height and hash are provided.
  ([\#3992](https://github.com/cometbft/cometbft/issues/3992))
