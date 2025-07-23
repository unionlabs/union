import type { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Effect } from "effect"
import type { Address } from "viem"

/**
 * Read ERC20 token metadata (name, symbol, decimals)
 * @param tokenAddress The address of the ERC20 token
 * @param chainId The Universal chain ID to check for gas denomination
 * @returns An Effect that resolves to the token metadata
 *
 * @category utils
 * @since 2.0.0
 */
export const readMeta = (tokenAddress: Address, chainId: UniversalChainId) =>
  Effect.gen(function*() {
    // Check if this is a gas denomination token for the specific chain
    const gasTokenMeta = GAS_DENOMS[chainId]

    if (gasTokenMeta && gasTokenMeta.address.toLowerCase() === tokenAddress.toLowerCase()) {
      // Return the metadata from GAS_DENOMS
      return {
        name: gasTokenMeta.name,
        symbol: gasTokenMeta.symbol,
        decimals: gasTokenMeta.decimals,
      }
    }

    // For regular ERC20 tokens, read from contract
    const name = yield* readName(tokenAddress)
    const symbol = yield* readSymbol(tokenAddress)
    const decimals = yield* readDecimals(tokenAddress)
    return { name, symbol, decimals }
  })

/**
 * Read the balance of an ERC20 token for a specific address
 *
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address to check the balance for
 *
 * @returns An Effect that resolves to the token balance
 *
 * @category utils
 * @since 2.0.0
 */
export const readBalance = (tokenAddress: Address, ownerAddress: Address) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "balanceOf",
      args: [ownerAddress],
    })
  })

/**
 * Read the balance of an ERC20 token for a specific address
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address to check the balance for
 * @param blockNumber The blockNumber at certain point
 * @returns An Effect that resolves to the token balance
 *
 * @category utils
 * @since 2.0.0
 */
export const readBalanceAtBlock = (
  tokenAddress: Address,
  ownerAddress: Address,
  blockNumber: bigint,
) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "balanceOf",
      args: [ownerAddress],
      blockNumber: blockNumber,
    })
  })

/**
 * Read the name of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token name
 *
 * @category utils
 * @since 2.0.0
 */
export const readName = (tokenAddress: Address) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "name",
    })
  })

/**
 * Read the symbol of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token symbol
 *
 * @category utils
 * @since 2.0.0
 */
export const readSymbol = (tokenAddress: Address) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "symbol",
    })
  })

/**
 * Read the decimals of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token decimals
 *
 * @category utils
 * @since 2.0.0
 */
export const readDecimals = (tokenAddress: Address) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "decimals",
    })
  })

/**
 * Read the TotalSupply of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @param blockNumber The blockNumber at certain point
 * @returns An Effect that resolves to the totalSupply
 *
 * @category utils
 * @since 2.0.0
 */
export const readTotalSupplyAtBlock = (tokenAddress: Address, blockNumber: bigint) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "totalSupply",
      blockNumber: blockNumber,
    })
  })

/**
 * Read the TotalSupply of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the totalSupply
 *
 * @category utils
 * @since 2.0.0
 */
export const readTotalSupply = (tokenAddress: Address) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "totalSupply",
    })
  })

/**
 * Read the allowance of an ERC20 token for a specific owner and spender
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address of the token owner
 * @param spenderAddress The address of the spender
 * @returns An Effect that resolves to the token allowance
 *
 * @category utils
 * @since 2.0.0
 */
export const readAllowance = (
  tokenAddress: Address,
  ownerAddress: Address,
  spenderAddress: Address,
) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    return yield* readContract(client, {
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "allowance",
      args: [ownerAddress, spenderAddress],
    })
  })

/**
 * Increase the allowance of an ERC20 token for a specific spender
 * @param tokenAddress The address of the ERC20 token
 * @param spenderAddress The address of the spender
 * @param amount The amount to increase the allowance by
 * @returns An Effect that resolves to the transaction hash
 *
 * @category utils
 * @since 2.0.0
 */
export const increaseAllowance = (
  tokenAddress: Address,
  spenderAddress: Address,
  amount: bigint,
) =>
  Effect.gen(function*() {
    const walletClient = yield* WalletClient

    return yield* writeContract({
      account: walletClient.account,
      chain: walletClient.chain,
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "approve",
      args: [spenderAddress, amount],
    })
  })
