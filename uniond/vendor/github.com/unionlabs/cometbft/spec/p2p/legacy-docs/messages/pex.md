---
order: 6
---

# Peer Exchange

## Channels

Pex has one channel. The channel identifier is listed below.

| Name       | Number |
|------------|--------|
| PexChannel | 0      |

## Message Types

The current PEX service has two versions. The first uses IP/port pair but since the p2p stack is moving towards a transport agnostic approach,
node endpoints require a `Protocol` and `Path` hence the V2 version uses a [url](https://golang.org/pkg/net/url/#URL) instead.

### PexRequest

PexRequest is an empty message requesting a list of peers.

> EmptyRequest

### PexResponse

PexResponse is an list of net addresses provided to a peer to dial.

| Name  | Type                               | Description                              | Field Number |
|-------|------------------------------------|------------------------------------------|--------------|
| addresses | repeated [PexAddress](#pexaddress) | List of peer addresses available to dial | 1            |

### PexAddress

PexAddress provides needed information for a node to dial a peer.

| Name | Type   | Description      | Field Number |
|------|--------|------------------|--------------|
| id   | string | NodeID of a peer | 1            |
| ip   | string | The IP of a node | 2            |
| port | port   | Port of a peer   | 3            |


### PexRequestV2

PexRequest is an empty message requesting a list of peers.

> EmptyRequest

### PexResponseV2

PexResponse is an list of net addresses provided to a peer to dial.

| Name  | Type                               | Description                              | Field Number |
|-------|------------------------------------|------------------------------------------|--------------|
| addresses | repeated [PexAddressV2](#pexresponsev2) | List of peer addresses available to dial | 1   |

### PexAddressV2

PexAddress provides needed information for a node to dial a peer.

| Name | Type   | Description      | Field Number |
|------|--------|------------------|--------------|
| url   | string | See [golang url](https://golang.org/pkg/net/url/#URL) | 1            |

### Message

Message is a [`oneof` protobuf type](https://developers.google.com/protocol-buffers/docs/proto#oneof). The one of consists of two messages.

| Name         | Type                      | Description                                          | Field Number |
|--------------|---------------------------|------------------------------------------------------|--------------|
| pex_request  | [PexRequest](#pexrequest) | Empty request asking for a list of addresses to dial | 1            |
| pex_response | [PexResponse](#pexresponse)| List of addresses to dial                           | 2            |
| pex_request_v2| [PexRequestV2](#pexrequestv2)| Empty request asking for a list of addresses to dial| 3         |
| pex_response_v2| [PexRespinseV2](#pexresponsev2)| List of addresses to dial 					  | 4 			 |
