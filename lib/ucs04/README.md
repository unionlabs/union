# UCS04

Rust implementation of [UCS04] Chain IDs. Union uses a two-part identifier — chain family name and chain ID — to uniquely identify a network. This helps prevent ambiguity across environments, testnets, and mainnets.

This crate also exports well known IDs, as defined below.

## `well-known.json`

This file defines all known universal chain IDs used in the Union ecosystem. It provides the canonical mapping from chain family to their supported chain IDs and serves as a reference for disambiguating chain identifiers across environments.

The structure is as follows:

- `<chain family name>`: The name of the chain family (e.g. `ethereum`, `babylon`).
  - A list of supported chain IDs for that chain. These IDs are used to form **universal chain IDs** by combining the chain name and ID, e.g. `ethereum.1`, `babylon.bbn-test-5`.

[ucs04]: https://docs.union.build/ucs/04/
