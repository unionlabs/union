---
title: "Permissionless versus Trustless"
---

In web3, many protocols are permissionless in some form or another. It can refer to being able to interact with the contracts directly, without requiring some form of authorization. Ethereum transfers are permissionless for example, as you do not need to be whitelisted to create a wallet and transfer coins. USDC is permissioned, but by default, anyone is allowed to transfer USDC. Wallets can be blacklisted and frozen by Circle, so the contracts are permissioned, and trusted by Circle with full admin rights. CCTP is built on the same principles, open by default, but fully trusted and permissioned.

## Bridging Protocols

For infrastructure such as bridges, most have the same characteristics as USDC; a small authority has control of bridged tokens and bridging transfers and can censor bridging transactions. It's thus trusted and permissioned. This means that the infrastructure provider can censor transactions, withhold funds, and print unbacked assets.

:::info
You can easily verify if a bridge can be censored by looking at the token contract. If the deposit/send function only emits an event, it is trusted and censorable:

- [Kava](https://github.com/Kava-Labs/kava-bridge/blob/3d88653f6c196a06d5b6f8abd5d03a679e0e030a/contract/contracts/Bridge.sol#L69)
- [Layerzero](https://github.com/LayerZero-Labs/wrapped-asset-bridge/blob/13c8582fc6492ff78966647c6ebd5913c192d602/contracts/WrappedTokenBridge.sol#L81)
- [Axelar](https://github.com/axelarnetwork/axelar-cgp-solidity/blob/9c7a260c848011f27d6e7ecb1cba88de79206ccc/contracts/AxelarGateway.sol#L103)

:::

Most infrastructure providers require permission to deploy on a new chain or rollup, you need to work with them to get contracts deployed and services operational; as well as that you are subject to being shut down at any moment. This is often the case for trusted protocols because the bridging provider takes a centralized role of authority.

### Permissionless but Trusted

An odd one out in the bunch is [Hyperlane](https://www.hyperlane.xyz/), which is permissionless but trusted. In their model, teams/parties run the centralized party themselves but are allowed to use Hyperlane contracts, relayers, and services. In this case, there is no risk of being shut down, but there are significant limitations:

- Not all assets are transferable.
- The bridge is still very susceptible to attacks.
- One party can still censor transactions.

## IBC and Union

Union provides both permissionless deployments and connections, while also being fully trustless. This means we

- Cannot censor transactions.
- Do not rely on Unionlabs (or any other party) as a centralized entity.
- Allow for any asset to be transferable.

For app chains and rollups; there is no compromise anymore. IBC grants fast, cheap, and secure infrastructure.
