---
title: "SDK"
---

# Solidity SDK

:::info

The reader is expected to be familiar with [**IBC**](https://ibcprotocol.org/) and [**Solidity**](https://docs.soliditylang.org/en/v0.8.21/).

:::

## Introduction

The [**IIBCModule**](https://github.com/unionlabs/union/blob/0b9833120051dcd87080a4f1ed61aa1886562190/evm/contracts/core/05-port/IIBCModule.sol) interface defines a set of functions that modules must implement to enable cross-chain communication over IBC. These functions are callbacks that handle various stages of the IBC handshake and packet processing.

:::caution

Every callback must only be callable by the IBC module. Protocol developers must ensure that an appropriate **`onlyIBC`** modifier decorates all the callbacks.

:::

Below is an explanation of each function:

### `onChanOpenInit`

This function is responsible for validating the relayer-chosen parameters during the channel initialization. It may also perform custom initialization logic. The function can return an error to abort the handshake if the chosen parameters are invalid. It takes the following parameters:

- `Order` enum: The order of packet sequencing (ORDERED or UNORDERED).
- `connectionHops`: An array of connection hop identifiers.
- `portId`: The port identifier.
- `channelId`: The channel identifier.
- `counterparty`: Data of the counterparty channel.
- `version`: The version string provided by the relayer.

### `onChanOpenTry`

Similar to `onChanOpenInit`, this function verifies relayer-chosen parameters and counterparty-chosen version. It may also perform custom TRY logic. It takes the following parameters:

- Same as `onChanOpenInit`, plus:
- `counterpartyVersion`: The version string chosen by the counterparty.

### `onChanOpenAck`

This function handles the acknowledgment of the channel opening. It may error to abort the handshake if the counterparty-selected version is invalid. It can also perform custom ACK logic. It takes the following parameters:

- `portId`: The port identifier.
- `channelId`: The channel identifier.
- `counterpartyVersion`: The version string chosen by the counterparty.

### `onChanOpenConfirm`

Handles the confirmation of the channel opening. It may perform custom CONFIRM logic and may error to abort the handshake. It takes the following parameters:

- `portId`: The port identifier.
- `channelId`: The channel identifier.

### `onChanCloseInit`

Handles the initiation of channel closure. It takes the following parameters:

- `portId`: The port identifier.
- `channelId`: The channel identifier.

### `onChanCloseConfirm`

Handles the confirmation of channel closure. It takes the following parameters:

- `portId`: The port identifier.
- `channelId`: The channel identifier.

### `onRecvPacket`

This function is responsible for processing incoming packets. It must return an acknowledgement as defined by the IBC specification. It takes the following parameters:

- `IbcCoreChannelV1Packet.Data`: Data of the received packet.
- `relayer`: Address of the relayer.

:::caution

The module MUST ensure that if the acknowledgement is successful, state changes are written; otherwise, they are discarded.

:::

### `onAcknowledgementPacket`

Handles the acknowledgement of a sent packet. It takes the following parameters:

- `IbcCoreChannelV1Packet.Data`: Data of the sent packet.
- `acknowledgement`: The acknowledgement data.
- `relayer`: Address of the relayer.

The application can decide whether to do further processing or not based on the acknowledgement.
