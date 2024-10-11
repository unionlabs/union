---
order: 3
---

# Genesis

The genesis file is the starting point of a chain. An application will populate the `app_state` field in the genesis with their required fields. CometBFT is not able to validate this section because it is unaware what application state consists of.

## Genesis Fields

- `genesis_time`: The genesis time is the time the blockchain started or will start. If nodes are started before this time they will sit idle until the time specified.
- `chain_id`: The chainid is the chain identifier. Every chain should have a unique identifier. When conducting a fork based upgrade, we recommend changing the chainid to avoid network or consensus errors.
- `initial_height`: This field is the starting height of the blockchain. When conducting a chain restart to avoid restarting at height 1, the network is able to start at a specified height.
- `consensus_params`
    - `block`
        - `max_bytes`: The max amount of bytes a block can be.
        - `max_gas`: The maximum amount of gas that a block can have.
        - `time_iota_ms`: This parameter has no value anymore in CometBFT.

- `evidence`
      - `max_age_num_blocks`: After this preset amount of blocks has passed a single piece of evidence is considered invalid
      - `max_age_duration`: After this preset amount of time has passed a single piece of evidence is considered invalid.
      - `max_bytes`: The max amount of bytes of all evidence included in a block.

> Note: For evidence to be considered invalid, evidence must be older than both `max_age_num_blocks` and `max_age_duration`

- `validator`
      - `pub_key_types`: Defines which curves are to be accepted as a valid validator consensus key. CometBFT supports ed25519, secp256k1 and bls12381.

- `version`
      - `app_version`: The version of the application. This is set by the application and is used to identify which version of the app a user should be using in order to operate a node.

- `validators`
    - This is an array of validators. This validator set is used as the starting validator set of the chain. This field can be empty, if the application sets the validator set in `InitChain`.

- `app_hash`: The applications state root hash. This field does not need to be populated at the start of the chain, the application may provide the needed information via `Initchain`.

- `app_state`: This section is filled in by the application and is unknown to CometBFT.
