---
sidebar_position: 1
---

# `x/nft`

## Contents

## Abstract

`x/nft` is an implementation of a Cosmos SDK module, per [ADR 43](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-043-nft-module.md), that allows you to create nft classification, create nft, transfer nft, update nft, and support various queries by integrating the module. It is fully compatible with the ERC721 specification.

* [Concept](#concepts)
    * [Class](#class)
    * [NFT](#nft)
*[State](#state)
*[Messages](#messages)
    * [MsgSend](#msgsend)
*[Events](#events)

## Concepts

### Class

`x/nft` module defines a struct `Class` to describe the common characteristics of a class of nft, under this class, you can create a variety of nft, which is equivalent to an erc721 contract for Ethereum. The design is defined in the [ADR 043](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-043-nft-module.md).

### NFT

The full name of NFT is Non-Fungible Tokens. Because of the irreplaceable nature of NFT, it means that it can be used to represent unique things. The nft implemented by this module is fully compatible with Ethereum ERC721 standard.

## State

### Class

Class is mainly composed of `id`, `name`, `symbol`, `description`, `uri`, `uri_hash`,`data` where `id` is the unique identifier of the class, similar to the Ethereum ERC721 contract address, the others are optional.

* Class: `0x01 | classID | -> ProtocolBuffer(Class)`

### NFT

NFT is mainly composed of `class_id`, `id`, `uri`, `uri_hash` and `data`. Among them, `class_id` and `id` are two-tuples that identify the uniqueness of nft, `uri` and `uri_hash` is optional, which identifies the off-chain storage location of the nft, and `data` is an Any type. Use Any chain of `x/nft` modules can be customized by extending this field

* NFT: `0x02 | classID | 0x00 | nftID |-> ProtocolBuffer(NFT)`

### NFTOfClassByOwner

NFTOfClassByOwner is mainly to realize the function of querying all nfts using classID and owner, without other redundant functions.

* NFTOfClassByOwner: `0x03 | owner | 0x00 | classID | 0x00 | nftID |-> 0x01`

### Owner

Since there is no extra field in NFT to indicate the owner of nft, an additional key-value pair is used to save the ownership of nft. With the transfer of nft, the key-value pair is updated synchronously.

* OwnerKey: `0x04 | classID | 0x00  | nftID |-> owner`

### TotalSupply

TotalSupply is responsible for tracking the number of all nfts under a certain class. Mint operation is performed under the changed class, supply increases by one, burn operation, and supply decreases by one.

* OwnerKey: `0x05 | classID |-> totalSupply`

## Messages

In this section we describe the processing of messages for the NFT module.

:::warning
The validation of `ClassID` and `NftID` is left to the app developer.  
The SDK does not provide any validation for these fields.
:::

### MsgSend

You can use the `MsgSend` message to transfer the ownership of nft. This is a function provided by the `x/nft` module. Of course, you can use the `Transfer` method to implement your own transfer logic, but you need to pay extra attention to the transfer permissions.

The message handling should fail if:

* provided `ClassID` does not exist.
* provided `Id` does not exist.
* provided `Sender` does not the owner of nft.

## Events

The nft module emits proto events defined in [the Protobuf reference](https://buf.build/cosmos/cosmos-sdk/docs/main:cosmos.nft.v1beta1).
