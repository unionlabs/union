import { Effect } from "effect"
import type { ChainInfo as KeplrChainInfo, FeeCurrency } from "@keplr-wallet/types"
import type { ChainInfo as LeapChainInfo } from "@leapwallet/types"
import type { Chain } from "$lib/schema/chain.ts"
import { GasPriceError, GetChainInfoError } from "$lib/services/transfer-cosmos"
import { keplrChainInfoMap, leapChainInfoMap } from "$lib/services/cosmos/chain-info/configs"
import type { CosmosWalletId } from "$lib/wallet/cosmos"

export const getCosmosChainInfo = (
  chain: Chain,
  connectedWallet: CosmosWalletId
): Effect.Effect<KeplrChainInfo | LeapChainInfo, GetChainInfoError, never> =>
  Effect.gen(function* () {
    if (!chain?.chain_id) {
      yield* Effect.fail(
        new GetChainInfoError({
          cause: "Invalid chain: missing chain_id"
        })
      )
      return null as never
    }

    if (!connectedWallet) {
      yield* Effect.fail(
        new GetChainInfoError({
          cause: "No wallet connected",
          chainId: chain.chain_id
        })
      )
      return null as never
    }

    const chainInfoMap = connectedWallet === "leap" ? leapChainInfoMap : keplrChainInfoMap

    const chainInfo = chainInfoMap[chain.chain_id]

    if (!chainInfo) {
      yield* Effect.fail(
        new GetChainInfoError({
          cause: `Chain info not found`,
          chainId: chain.chain_id
        })
      )
      return null as never
    }

    return chainInfo
  })

export const getHighGasPriceStep = (
  chainInfo: KeplrChainInfo | LeapChainInfo
): Effect.Effect<{ amount: string; denom: string }, GasPriceError, never> =>
  Effect.gen(function* () {
    if (!Array.isArray(chainInfo.feeCurrencies) || chainInfo.feeCurrencies.length === 0) {
      yield* Effect.fail(
        new GasPriceError({
          cause: "No fee currencies defined in chain info",
          chainId: chainInfo.chainId
        })
      )
      return null as never
    }

    const feeCurrency = chainInfo.feeCurrencies[0] as FeeCurrency

    if (!feeCurrency.gasPriceStep) {
      yield* Effect.fail(
        new GasPriceError({
          cause: "Gas price step not defined for fee currency",
          chainId: chainInfo.chainId
        })
      )
      return null as never
    }

    return {
      amount: feeCurrency.gasPriceStep.high.toString(),
      denom: feeCurrency.coinMinimalDenom
    }
  })

export const getGasPriceForChain = (
  chain: Chain,
  connectedWallet: CosmosWalletId
): Effect.Effect<{ amount: number; denom: string }, GetChainInfoError | GasPriceError, never> =>
  Effect.gen(function* () {
    const chainInfo = yield* getCosmosChainInfo(chain, connectedWallet)
    console.log(chainInfo)
    const gasPriceStep = yield* getHighGasPriceStep(chainInfo)
    console.log(gasPriceStep)

    return {
      amount: Number.parseFloat(gasPriceStep.amount),
      denom: gasPriceStep.denom
    }
  })

export const isNativeToken = (
  token: string,
  chain: Chain,
  connectedWallet: CosmosWalletId
): Effect.Effect<boolean, GetChainInfoError, never> =>
  Effect.gen(function* () {
    const chainInfo = yield* getCosmosChainInfo(chain, connectedWallet)

    if (Array.isArray(chainInfo.feeCurrencies)) {
      for (const feeCurrency of chainInfo.feeCurrencies) {
        if (feeCurrency.coinDenom === token || feeCurrency.coinMinimalDenom === token) {
          return true
        }
      }
    }

    if (
      chainInfo.stakeCurrency &&
      (chainInfo.stakeCurrency.coinDenom === token ||
        chainInfo.stakeCurrency.coinMinimalDenom === token)
    ) {
      return true
    }

    // Not a native token
    return false
  })
