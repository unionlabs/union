import { Effect } from "effect"
import { erc20Abi, type PublicClient, type ReadContractErrorType } from "viem"
import { ReadContractError } from "./balances"
import type { TokenRawDenom } from "$lib/schema/token"

export const fetchErc20Name = ({
  client,
  tokenAddress
}: {
  client: PublicClient
  tokenAddress: TokenRawDenom
}) =>
  Effect.tryPromise({
    try: () =>
      client.readContract({
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "name"
      }),
    catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
  })

export const fetchErc20Symbol = ({
  client,
  tokenAddress
}: {
  client: PublicClient
  tokenAddress: TokenRawDenom
}) =>
  Effect.tryPromise({
    try: () =>
      client.readContract({
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "symbol"
      }),
    catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
  })

export const fetchErc20Decimals = ({
  client,
  tokenAddress
}: {
  client: PublicClient
  tokenAddress: TokenRawDenom
}) =>
  Effect.tryPromise({
    try: () =>
      client.readContract({
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "decimals"
      }),
    catch: err => new ReadContractError({ cause: err as ReadContractErrorType })
  })
