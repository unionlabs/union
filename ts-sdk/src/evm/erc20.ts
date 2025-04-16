import { Effect } from "effect"
import { erc20Abi, type Address } from "viem"
import { readContract } from "./contract.js"
import { writeContract } from "./contract.js"
import { ViemPublicClient, ViemWalletClient } from "./client.js"

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
 * Read the balance of an ERC20 token for a specific address
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address to check the balance for
 * @returns An Effect that resolves to the token balance
 */
export const readErc20Balance = (tokenAddress: Address, ownerAddress: Address) =>
  Effect.gen(function* () {
    const client = (yield* ViemPublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "balanceOf",
      args: [ownerAddress]
    })
  })

/**
 * Read the name of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token name
 */
export const readErc20Name = (tokenAddress: Address) =>
  Effect.gen(function* () {
    const client = (yield* ViemPublicClient).client

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
    const client = (yield* ViemPublicClient).client

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
    const client = (yield* ViemPublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "decimals"
    })
  })

/**
 * Read the allowance of an ERC20 token for a specific owner and spender
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address of the token owner
 * @param spenderAddress The address of the spender
 * @returns An Effect that resolves to the token allowance
 */
export const readErc20Allowance = (
  tokenAddress: Address,
  ownerAddress: Address,
  spenderAddress: Address
) =>
  Effect.gen(function* () {
    const client = (yield* ViemPublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "allowance",
      args: [ownerAddress, spenderAddress]
    })
  })

/**
 * Increase the allowance of an ERC20 token for a specific spender
 * @param tokenAddress The address of the ERC20 token
 * @param spenderAddress The address of the spender
 * @param amount The amount to increase the allowance by
 * @returns An Effect that resolves to the transaction hash
 */
export const increaseErc20Allowance = (
  tokenAddress: Address,
  spenderAddress: Address,
  amount: bigint
) =>
  Effect.gen(function* () {
    const walletClient = yield* ViemWalletClient

    return yield* writeContract(walletClient.client, {
      account: walletClient.account,
      chain: walletClient.chain,
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "approve",
      args: [spenderAddress, amount]
    })
  })
