---
order: 3
---

# Evidence

## Channel

Evidence has one channel. The channel identifier is listed below.

| Name            | Number |
|-----------------|--------|
| EvidenceChannel | 56     |

## Message Types

### EvidenceList

EvidenceList consists of a list of verified evidence. This evidence will already have been propagated throughout the network. EvidenceList is used in two places, as a p2p message and within the block [block](../../../core/data_structures.md#block) as well.

| Name     | Type                                                        | Description            | Field Number |
|----------|-------------------------------------------------------------|------------------------|--------------|
| evidence | repeated [Evidence](../../../core/data_structures.md#evidence) | List of valid evidence | 1            |
