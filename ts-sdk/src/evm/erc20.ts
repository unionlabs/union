import { Effect } from "effect"
import { erc20Abi, type Address } from "viem"
import { readContract } from "./contract.js"
import { PublicViemClient } from "./client.js"

/**
 * Read ERC20 token metadata (name, symbol, decimals)
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token metadata
 */
export const readErc20Meta = (tokenAddress: Address) =>
  Effect.gen(function* () {
    const name = yield* readErc20Name(tokenAddress)
    const symbol = yield* readErc20Symbol(tokenAddress)
    const decimals = yield* readErc20Decimals(tokenAddress)
    return { name, symbol, decimals }
  })

/**
 * Read the name of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token name
 */
export const readErc20Name = (tokenAddress: Address) =>
  Effect.gen(function* () {
    const client = (yield* PublicViemClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "name"
    })
  })

/**
 * Read the symbol of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token symbol
 */
export const readErc20Symbol = (tokenAddress: Address) =>
  Effect.gen(function* () {
    const client = (yield* PublicViemClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "symbol"
    })
  })

/**
 * Read the decimals of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token decimals
 */
export const readErc20Decimals = (tokenAddress: Address) =>
  Effect.gen(function* () {
    const client = (yield* PublicViemClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "decimals"
    })
  })
