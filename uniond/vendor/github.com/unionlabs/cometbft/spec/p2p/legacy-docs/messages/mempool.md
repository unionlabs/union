---
order: 4
---
# Mempool

## Channel

Mempool has one channel. The channel identifier is listed below.

| Name           | Number |
|----------------|--------|
| MempoolChannel | 48     |

## Message Types

There is currently only one message that Mempool broadcasts and receives over
the p2p gossip network (via the reactor): `Txs`

### Txs

A list of transactions. These transactions have been checked against the application for validity. This does not mean that the transactions are valid, it is up to the application to check this.

| Name | Type           | Description          | Field Number |
|------|----------------|----------------------|--------------|
| txs  | repeated bytes | List of transactions | 1            |

### Message

Message is a [`oneof` protobuf type](https://developers.google.com/protocol-buffers/docs/proto#oneof). The one of consists of one message [`Txs`](#txs).

| Name | Type        | Description           | Field Number |
|------|-------------|-----------------------|--------------|
| txs  | [Txs](#txs) | List of transactions | 1            |
