import { GasPriceError, GetChainInfoError } from "$lib/services/transfer-ucs03-cosmos"
import type { ChainInfo as KeplrChainInfo, FeeCurrency } from "@keplr-wallet/types"
import type { ChainInfo as LeapChainInfo } from "@leapwallet/types"
import type { Chain } from "@unionlabs/sdk/schema"
import { Effect, Record as R } from "effect"
import { chainInfoMap } from "./config"
import type { InternalChainInfo } from "./internal-chain-info"

export const getCosmosChainInfo = (
  chain: Chain,
): Effect.Effect<InternalChainInfo, GetChainInfoError, never> =>
  Effect.gen(function*() {
    const chainInfo = yield* R.get(chainInfoMap, chain.chain_id).pipe(
      Effect.mapError(() =>
        new GetChainInfoError({
          cause: `Chain with ID ${chain.chain_id} is not configured.`,
          chainId: chain.chain_id,
        })
      ),
    )

    return chainInfo
  })

export const getHighGasPriceStep = (
  chainInfo: KeplrChainInfo | LeapChainInfo,
): Effect.Effect<{ amount: string; denom: string }, GasPriceError, never> =>
  Effect.gen(function*() {
    if (
      !Array.isArray(chainInfo.feeCurrencies)
      || chainInfo.feeCurrencies.length === 0
    ) {
      return yield* Effect.fail(
        new GasPriceError({
          cause: "No fee currencies defined in chain info",
          chainId: chainInfo.chainId,
        }),
      )
    }

    const feeCurrency = chainInfo.feeCurrencies[0] as FeeCurrency

    if (!feeCurrency.gasPriceStep) {
      return yield* Effect.fail(
        new GasPriceError({
          // TODO: change to `message`
          // TODO: reserve `cause` for originally thrown error
          cause: "Gas price step not defined for fee currency",
          chainId: chainInfo.chainId,
        }),
      )
    }

    return {
      amount: feeCurrency.gasPriceStep.high.toString(),
      denom: feeCurrency.coinMinimalDenom,
    }
  })

export const getGasPriceForChain = (
  chain: Chain,
): Effect.Effect<
  { amount: number; denom: string },
  GetChainInfoError | GasPriceError,
  never
> =>
  Effect.gen(function*() {
    const chainInfo = yield* getCosmosChainInfo(chain)
    const gasPriceStep = yield* getHighGasPriceStep(chainInfo)

    return {
      amount: Number.parseFloat(gasPriceStep.amount),
      denom: gasPriceStep.denom,
    }
  })

export const isNativeToken = (
  token: string,
  chain: Chain,
): Effect.Effect<boolean, GetChainInfoError, never> =>
  Effect.gen(function*() {
    const chainInfo = yield* getCosmosChainInfo(chain)

    if (Array.isArray(chainInfo.feeCurrencies)) {
      for (const feeCurrency of chainInfo.feeCurrencies) {
        if (
          feeCurrency.coinDenom === token
          || feeCurrency.coinMinimalDenom === token
        ) {
          return true
        }
      }
    }

    if (
      chainInfo.stakeCurrency
      && (chainInfo.stakeCurrency.coinDenom === token
        || chainInfo.stakeCurrency.coinMinimalDenom === token)
    ) {
      return true
    }

    // Not a native token
    return false
  })
