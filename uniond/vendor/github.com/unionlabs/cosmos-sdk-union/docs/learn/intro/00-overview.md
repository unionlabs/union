---
sidebar_position: 1
---

# High-level Overview

## What is the Cosmos SDK

The [Cosmos SDK](https://github.com/cosmos/cosmos-sdk) is an open-source toolkit for building multi-asset public Proof-of-Stake (PoS) <df value="blockchain">blockchains</df>, like the Cosmos Hub, as well as permissioned Proof-of-Authority (PoA) blockchains. Blockchains built with the Cosmos SDK are generally referred to as **application-specific blockchains**.

The goal of the Cosmos SDK is to allow developers to easily create custom blockchains from scratch that can natively interoperate with other blockchains. 
We further this modular approach by allowing developers to plug and play with different consensus engines this can range from [CometBFT](https://cometbft.com/) or [Rollkit](https://rollkit.dev/). 

SDK-based blockchains have the choice to use the predefined modules or to build their own modules. What this means is that developers can build a blockchain that is tailored to their specific use case, without having to worry about the low-level details of building a blockchain from scratch. Predefined modules include staking, governance, and token issuance, among others.

What's more, the Cosmos SDK is a capabilities-based system that allows developers to better reason about the security of interactions between modules. For a deeper look at capabilities, jump to [Object-Capability Model](../advanced/10-ocap.md).

You can think of the SDK as a lego kit. You can choose to build the basic house from the instructions, or you can choose to modify your house and add more floors, more doors, more windows. The choice is yours.

## What are Application-Specific Blockchains

One development paradigm in the blockchain world today is that of virtual-machine blockchains like Ethereum, where development generally revolves around building decentralized applications on top of an existing blockchain as a set of smart contracts. While smart contracts can be very good for some use cases like single-use applications (e.g. ICOs), they often fall short for building complex decentralized platforms. More generally, smart contracts can be limiting in terms of flexibility, sovereignty and performance.

Application-specific blockchains offer a radically different development paradigm than virtual-machine blockchains. An application-specific blockchain is a blockchain customized to operate a single application: developers have all the freedom to make the design decisions required for the application to run optimally. They can also provide better sovereignty, security and performance.

Learn more about [application-specific blockchains](./01-why-app-specific.md).

## What is Modularity

Today, there is a lot of talk around modularity and discussions between monolithic and modular. Originally, the Cosmos SDK was built with a vision of modularity in mind. Modularity is derived from splitting a blockchain into customizable layers of execution, consensus, settlement and data availability, which is what the Cosmos SDK enables. This means that developers can plug and play, making their blockchain customizable by using different software for different layers. For example, you can choose to build a vanilla chain and use the Cosmos SDK with CometBFT. CometBFT will be your consensus layer and the chain itself would be the settlement and execution layer. Another route could be to use the SDK with Rollkit and [Celestia](https://celestia.org/) as your consensus and data availability layer. The benefit of modularity is that you can customize your chain to your specific use case.

## Why the Cosmos SDK

The Cosmos SDK is the most advanced framework for building custom modular application-specific blockchains today. Here are a few reasons why you might want to consider building your decentralized application with the Cosmos SDK:

* It allows you to plug and play and customize your consensus layer. As mentioned above, you can use Rollkit and Celestia as your consensus and data availability layer. This offers a lot of flexibility and customization. 
* Previously the default consensus engine available within the Cosmos SDK is [CometBFT](https://cometbft.com/). CometBFT is the most (and only) mature BFT consensus engine in existence. It is widely used across the industry and is considered the gold standard consensus engine for building Proof-of-Stake systems.
* The Cosmos SDK is open-source and designed to make it easy to build blockchains out of composable [modules](../../build/modules). As the ecosystem of open-source Cosmos SDK modules grows, it will become increasingly easier to build complex decentralized platforms with it.
* The Cosmos SDK is inspired by capabilities-based security, and informed by years of wrestling with blockchain state-machines. This makes the Cosmos SDK a very secure environment to build blockchains.
* Most importantly, the Cosmos SDK has already been used to build many application-specific blockchains that are already in production. Among others, we can cite [Cosmos Hub](https://hub.cosmos.network), [Osmosis](https://osmosis.zone/), [Binance Chain](https://docs.binance.org/), [Terra](https://terra.money/) or [Dydx](https://dydx.exchange/). [Many more](https://cosmos.network/ecosystem) are building on the Cosmos SDK.

## Getting started with the Cosmos SDK

* Learn more about the [architecture of a Cosmos SDK application](./02-sdk-app-architecture.md)
* Learn how to build an application-specific blockchain from scratch with the [Cosmos SDK Tutorial](https://cosmos.network/docs/tutorial)
