---
title: "PingPong"
---

## Overview

The [**`PingPong`**](https://github.com/unionlabs/union/blob/52d586d250bab396f7a07b03bf77ffb2fb459365/evm/contracts/apps/ucs/00-pingpong/PingPong.sol) contract implements a simple ping pong protocol that alternates between sending "ping" and "pong" messages between two blockchain networks connected via the IBC protocol.

### Contract Structure

The contract consists of the following components:

1. **Data Structures and Library**: Defines a `PingPongPacket` struct to represent the ping pong packet data and a library `PingPongPacketLib` for encoding and decoding packets.

2. **Contract Definition**: The `PingPong` contract inherits from `IBCAppBase` and implements  the IBC callbacks.

3. **Constructor**: Initializes the contract with the IBC handler, revision number, and the number of blocks before pong timeout.

4. **IBCAppBase Overrides**: Overrides functions required by the IBC protocol, including `ibcAddress`, `onRecvPacket`, and channel open/close callbacks.

5. **Initiation**: The `initiate` function sends a ping or pong packet to the counterparty chain.

## Protocol Workflow

1. The contract is deployed with the IBC handler, revision number, and timeout information.
2. One side of the channel initiates the ping pong protocol by calling the `initiate` function.
3. Upon receiving a packet, the `onRecvPacket` function is triggered, emitting a `Ring` event and sending a response packet to the counterparty chain.
4. The protocol continues to alternate between ping and pong messages as each side of the channel processes incoming packets.

## Conclusion

The `PingPong` contract showcases a basic example of cross-chain communication over the IBC protocol using Solidity. It demonstrates how developers can implement a simple ping pong protocol between two blockchain networks, exchanging packets and alternating between ping and pong messages. This example serves as a starting point for building more complex cross-chain communication scenarios and applications using IBC and Solidity.
