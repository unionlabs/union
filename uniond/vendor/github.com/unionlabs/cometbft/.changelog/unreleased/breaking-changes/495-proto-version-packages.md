- `[proto]` Renamed the packages from `tendermint.*` to `cometbft.*`
  and introduced versioned packages to distinguish between proto definitions
  released in `0.34.x`, `0.37.x`, `0.38.x`, and `1.x` versions.
  Prior to the 1.0 release, the versioned packages are suffixed with
  `.v1beta1`, `.v1beta2`, and so on; all definitions describing the protocols
  as per the 1.0.0 release are in packages suffixed with `.v1`.
  Relocated generated Go code into a new `api` folder and changed the import
  paths accordingly.
  ([\#495](https://github.com/cometbft/cometbft/pull/495),
  [\#1504](https://github.com/cometbft/cometbft/issues/1504))
