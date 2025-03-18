import { Effect } from "effect"
import { erc20Abi, type PublicClient, type ReadContractErrorType } from "viem"
import { ReadContractError } from "./balances"
import type { TokenRawDenom } from "$lib/schema/token"
import { PublicSourceViemClient } from "./clients"

export const readErc20Meta = (tokenAddress: TokenRawDenom) =>
  Effect.gen(function* () {
    const publicClient = (yield* PublicSourceViemClient).client
    const name = yield* readErc20Name(tokenAddress)
    const symbol = yield* readErc20Symbol(tokenAddress)
    const decimals = yield* readErc20Decimals(tokenAddress)
    return { name, symbol, decimals }
  })

export const readErc20Name = (tokenAddress: TokenRawDenom) =>
  Effect.gen(function* () {
    const publicClient = (yield* PublicSourceViemClient).client
    const name = yield* Effect.tryPromise({
      try: () =>
        publicClient.readContract({
          address: tokenAddress,
          abi: erc20Abi,
          functionName: "name"
        }),
      catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
    })

    return name
  })

export const readErc20Symbol = (tokenAddress: TokenRawDenom) =>
  Effect.gen(function* () {
    const publicClient = (yield* PublicSourceViemClient).client
    const symbol = yield* Effect.tryPromise({
      try: () =>
        publicClient.readContract({
          address: tokenAddress,
          abi: erc20Abi,
          functionName: "symbol"
        }),
      catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
    })

    return symbol
  })

export const readErc20Decimals = (tokenAddress: TokenRawDenom) =>
  Effect.gen(function* () {
    const publicClient = (yield* PublicSourceViemClient).client
    const decimals = yield* Effect.tryPromise({
      try: () =>
        publicClient.readContract({
          address: tokenAddress,
          abi: erc20Abi,
          functionName: "decimals"
        }),
      catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
    })

    return decimals
  })
