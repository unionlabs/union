import { FetchHttpClient } from "@effect/platform"
import { Effect } from "effect"
import { AddressCosmosDisplay } from "../schema/address.js"
import { CosmWasmClientContext, SigningCosmWasmClientContext } from "./client.js"
import { executeContract, queryContract } from "./contract.js"
import { queryContractSmartAtHeight } from "./query.js"

/**
 * Interface for CW20 token metadata
 */
export interface Cw20TokenInfo {
  name: string
  symbol: string
  decimals: number
  total_supply: string
}

/**
 * Interface for CW20 token balance response
 */
export interface Cw20BalanceResponse {
  balance: string
}

/**
 * Interface for CW20 token balance response
 */
export interface Cw20AllowanceResponse {
  allowance: number
  expiration: any
}

/**
 * Read CW20 token metadata (name, symbol, decimals, total_supply)
 * @param contractAddress The address of the CW20 token contract
 * @returns An Effect that resolves to the token metadata
 */
export const readCw20TokenInfo = (contractAddress: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client

    return yield* queryContract<Cw20TokenInfo>(client, contractAddress, { token_info: {} })
  })

/**
 * Read CW20 token total_supply
 * @param contractAddress The address of the CW20 token contract
 * @returns An Effect that resolves to the token total supply
 */
export const readCw20TotalSupply = (contractAddress: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client
    const token_info = yield* queryContract<Cw20TokenInfo>(client, contractAddress, {
      token_info: {},
    })
    return token_info.total_supply
  })

/**
 * Read the balance of a CW20 token for a specific address
 * @param rest The rest url
 * @param contractAddress The address of the CW20 token contract
 * @param address The address to check the balance for
 * @param height Height of the chain
 * @returns An Effect that resolves to the token balance
 */
export const readCw20BalanceAtHeight = (
  rest: string,
  contractAddress: string,
  address: string,
  height: number,
) =>
  Effect.gen(function*() {
    const resp = yield* queryContractSmartAtHeight<{ data: { balance: string } }>(
      rest,
      contractAddress,
      {
        balance: {
          address,
        },
      },
      height,
    ).pipe(
      Effect.provide(FetchHttpClient.layer),
      Effect.tapErrorCause((cause) => Effect.logError("cosmos.readCw20BalanceAtHeight", cause)),
    )
    return resp.data.balance
  })

/**
 * Read CW20 token total_supply
 * @param rest The rest url
 * @param contractAddress The address of the CW20 token contract
 * @param height Height of the chain
 * @returns An Effect that resolves to the token total supply
 */
export const readCw20TotalSupplyAtHeight = (
  rest: string,
  contractAddress: string,
  height: number,
) =>
  Effect.gen(function*() {
    const resp = yield* queryContractSmartAtHeight<
      { data: { name: string; symbol: string; decimals: number; total_supply: string } }
    >(rest, contractAddress, {
      token_info: {},
    }, height).pipe(
      Effect.provide(FetchHttpClient.layer),
      Effect.tapErrorCause((cause) => Effect.logError("cosmos.readCw20TotalSupplyAtHeight", cause)),
    )
    return resp.data.total_supply
  })

/**
 * Read the balance of a CW20 token for a specific address
 * @param contractAddress The address of the CW20 token contract
 * @param address The address to check the balance for
 * @returns An Effect that resolves to the token balance
 */
export const readCw20Balance = (contractAddress: string, address: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client

    const response = yield* queryContract<Cw20BalanceResponse>(client, contractAddress, {
      balance: {
        address,
      },
    })

    return response.balance
  })

/**
 * Read the allowance of a CW20 token for a specific addresses
 * @param contract The address of the CW20 token contract
 * @param owner The owner of the token
 * @param spender The spender who will spend the token
 * @returns An Effect that resolves to the token allowance
 */
export const readCw20Allowance = (
  contract: AddressCosmosDisplay,
  owner: AddressCosmosDisplay,
  spender: AddressCosmosDisplay,
) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client

    const response = yield* queryContract<Cw20AllowanceResponse>(client, contract, {
      allowance: {
        owner: owner,
        spender: spender,
      },
    })

    return response.allowance
  })

/**
 * Increase the allowance of a CW20 token for a specific spender.
 *
 * @param contractAddress The address of the CW20 token contract.
 * @param senderAddress The address of the token owner (the one increasing the allowance).
 * @param spenderAddress The address of the spender who is allowed to spend the tokens.
 * @param amount The amount by which to increase the allowance (as a string).
 * @param expires Optional expiration for the increased allowance.
 * @returns An Effect that resolves to the execution result.
 */
export const writeCw20IncreaseAllowance = (
  contractAddress: string,
  senderAddress: string,
  spenderAddress: string,
  amount: string,
) =>
  Effect.gen(function*() {
    const client = (yield* SigningCosmWasmClientContext).client

    return yield* executeContract(client, senderAddress, contractAddress, {
      increase_allowance: {
        spender: spenderAddress,
        amount,
      },
    })
  })

/**
 * Checks whether a denom is a native token or CW20.
 * @param denom The denom address to check.
 * @returns An Effect that resolves to true if native, false if CW20.
 */
export const isDenomNative = (denom: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client

    return yield* readCw20TokenInfo(denom).pipe(
      Effect.provideService(CosmWasmClientContext, { client }),
      Effect.map(() => false),
      Effect.catchAllCause(() => Effect.succeed(true)),
    )
  })
