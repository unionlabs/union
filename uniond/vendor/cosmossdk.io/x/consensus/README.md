---
sidebar_position: 1
---

# `x/consensus`

## Abstract 

Functionality to modify CometBFT's ABCI consensus params.

## Contents

* [Abstract](#abstract)
* [Contents](#contents)
* [State](#state)
* [Params](#params)
* [Keeper](#keeper)
* [Messages](#messages)
    * [UpdateParams](#updateparams)
* [Events](#events)

## State

The `x/consensus` module keeps state of the consensus params from CometBFT.

## Params

The consensus module stores its params in state with the prefix of `0x05`,
it can be updated with governance or the address with authority.

* Params: `0x05 | ProtocolBuffer(cometbft.ConsensusParams)`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/consensus/proto/cosmos/consensus/v1/query.proto#L21-L27
```

```protobuf reference
https://github.com/cometbft/cometbft/blob/v0.34.35/proto/tendermint/types/params.proto#L11-L18
```

## Keeper

The Keeper of the `x/consensus` module provides the following functions:

* `Params`: Retrieves the current consensus parameters.

* `UpdateParams`: Updates the consensus parameters. Only the authority can perform this operation.

* `BlockParams`: Returns the maximum gas and bytes allowed in a block.

* `ValidatorPubKeyTypes`: Provides the list of public key types allowed for validators.

* `EvidenceParams`: Returns the evidence parameters, including maximum age and bytes.

* `AppVersion`: Returns the current application version.


Note: It is recommended to use the `x/consensus` module keeper to get consensus params instead of accessing them through the context.


## Messages

### UpdateParams

Update consensus params.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/consensus/proto/cosmos/consensus/v1/tx.proto#L24-L44
```

The message will fail under the following conditions:

* The signer is not the set authority 
* Not all values are set

## Events

The consensus module emits the following events:

### Message Events

#### MsgUpdateParams

| Type   | Attribute Key | Attribute Value     |
|--------|---------------|---------------------|
| string | authority     | msg.Signer          |
| string | parameters    | consensus Parameters |
