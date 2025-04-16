import { Effect } from "effect"
import { CosmWasmClientContext, SigningCosmWasmClientContext } from "./client.js"
import { queryContract, executeContract } from "./contract.js"

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
  Effect.gen(function* () {
    const client = (yield* CosmWasmClientContext).client

    return yield* queryContract<Cw20TokenInfo>(client, contractAddress, { token_info: {} })
  })

/**
 * Read the balance of a CW20 token for a specific address
 * @param contractAddress The address of the CW20 token contract
 * @param address The address to check the balance for
 * @returns An Effect that resolves to the token balance
 */
export const readCw20Balance = (contractAddress: string, address: string) =>
  Effect.gen(function* () {
    const client = (yield* CosmWasmClientContext).client

    const response = yield* queryContract<Cw20BalanceResponse>(client, contractAddress, {
      balance: {
        address
      }
    })

    return response.balance
  })

/**
 * Read the allowance of a CW20 token for a specific addresses
 * @param contractAddress The address of the CW20 token contract
 * @param ownerAddress The owner of the token
 * @param spenderAddress The spender who will spend the token
 * @returns An Effect that resolves to the token allowance
 */
export const readCw20Allowance = (
  contractAddress: string,
  ownerAddress: string,
  spenderAddress: string
) =>
  Effect.gen(function* () {
    const client = (yield* CosmWasmClientContext).client

    const response = yield* queryContract<Cw20AllowanceResponse>(client, contractAddress, {
      allowance: {
        owner: ownerAddress,
        spender: spenderAddress
      }
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
  amount: string
) =>
  Effect.gen(function* () {
    const client = (yield* SigningCosmWasmClientContext).client

    return yield* executeContract(client, senderAddress, contractAddress, {
      increase_allowance: {
        spender: spenderAddress,
        amount
      }
    })
  })
