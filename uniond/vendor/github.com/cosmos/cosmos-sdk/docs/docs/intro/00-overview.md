---
sidebar_position: 1
---

# High-level Overview

## What is the Cosmos SDK

The [Cosmos SDK](https://github.com/cosmos/cosmos-sdk) is an open-source framework for building multi-asset public Proof-of-Stake (PoS) <df value="blockchain">blockchains</df>, like the Cosmos Hub, as well as permissioned Proof-of-Authority (PoA) blockchains. Blockchains built with the Cosmos SDK are generally referred to as **application-specific blockchains**.

The goal of the Cosmos SDK is to allow developers to easily create custom blockchains from scratch that can natively interoperate with other blockchains. We envision the Cosmos SDK as the npm-like framework to build secure blockchain applications on top of [CometBFT](https://github.com/cometbft/cometbft). SDK-based blockchains are built out of composable [modules](../building-modules/01-intro.md), most of which are open-source and readily available for any developers to use. Anyone can create a module for the Cosmos SDK, and integrating already-built modules is as simple as importing them into your blockchain application. What's more, the Cosmos SDK is a capabilities-based system that allows developers to better reason about the security of interactions between modules. For a deeper look at capabilities, jump to [Object-Capability Model](../core/10-ocap.md).

## What are Application-Specific Blockchains

One development paradigm in the blockchain world today is that of virtual-machine blockchains like Ethereum, where development generally revolves around building decentralized applications on top of an existing blockchain as a set of smart contracts. While smart contracts can be very good for some use cases like single-use applications (e.g. ICOs), they often fall short for building complex decentralized platforms. More generally, smart contracts can be limiting in terms of flexibility, sovereignty and performance.

Application-specific blockchains offer a radically different development paradigm than virtual-machine blockchains. An application-specific blockchain is a blockchain customized to operate a single application: developers have all the freedom to make the design decisions required for the application to run optimally. They can also provide better sovereignty, security and performance.

Learn more about [application-specific blockchains](./01-why-app-specific.md).

## Why the Cosmos SDK

The Cosmos SDK is the most advanced framework for building custom application-specific blockchains today. Here are a few reasons why you might want to consider building your decentralized application with the Cosmos SDK:

* The default consensus engine available within the Cosmos SDK is [CometBFT](https://github.com/cometbft/cometbft). CometBFT is the most (and only) mature BFT consensus engine in existence. It is widely used across the industry and is considered the gold standard consensus engine for building Proof-of-Stake systems.
* The Cosmos SDK is open-source and designed to make it easy to build blockchains out of composable [modules](../modules). As the ecosystem of open-source Cosmos SDK modules grows, it will become increasingly easier to build complex decentralized platforms with it.
* The Cosmos SDK is inspired by capabilities-based security, and informed by years of wrestling with blockchain state-machines. This makes the Cosmos SDK a very secure environment to build blockchains.
* Most importantly, the Cosmos SDK has already been used to build many application-specific blockchains that are already in production. Among others, we can cite [Cosmos Hub](https://hub.cosmos.network), [IRIS Hub](https://irisnet.org), [Binance Chain](https://docs.binance.org/), [Terra](https://terra.money/) or [Kava](https://www.kava.io/). [Many more](https://cosmos.network/ecosystem) are building on the Cosmos SDK.

## Getting started with the Cosmos SDK

* Learn more about the [architecture of a Cosmos SDK application](./02-sdk-app-architecture.md)
* Learn how to build an application-specific blockchain from scratch with the [Cosmos SDK Tutorial](https://cosmos.network/docs/tutorial)
