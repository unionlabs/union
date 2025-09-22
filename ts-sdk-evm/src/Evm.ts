/**
 * This module handles EVM related functionality.
 *
 * @since 0.0.0
 */
import { GAS_DENOMS } from "@unionlabs/sdk/Constants"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import * as Utils from "@unionlabs/sdk/Utils"
import { Context, Data, Effect, flow, Layer, pipe, Schema as S } from "effect"
import * as O from "effect/Option"
import { type Address, erc20Abi } from "viem"
import {
  type Abi,
  type Account as ViemAccount,
  type Chain as ViemChain,
  type ContractFunctionArgs,
  type ContractFunctionName,
  type CreatePublicClientErrorType,
  type CreateWalletClientErrorType,
  type PublicClient as ViemPublicClient,
  type ReadContractErrorType,
  type ReadContractParameters,
  type WalletClient as ViemWalletClient,
  type WriteContractErrorType,
  type WriteContractParameters,
} from "viem"
import type {
  DeriveChain,
  Hash,
  SimulateContractErrorType,
  SimulateContractParameters,
  WaitForTransactionReceiptTimeoutErrorType,
} from "viem"
import type { Hex } from "viem"
import * as internal from "./internal/evm.js"
import * as Safe from "./Safe.js"

import type {} from "effect"

/**
 * @category models
 * @since 0.0.0
 */
export namespace Evm {
  /**
   * @category models
   * @since 0.0.0
   */
  export interface PublicClient {
    readonly client: ViemPublicClient
  }

  /**
   * @category models
   * @since 0.0.0
   */
  export interface WalletClient {
    readonly client: ViemWalletClient
    readonly account: ViemAccount
    readonly chain: ViemChain
  }

  /**
   * @category models
   * @since 0.0.0
   */
  export interface Channel {
    readonly ucs03address: Address
    readonly channelId: number
  }
}

/**
 * @category utils
 * @since 0.0.0
 */
export const channelBalance = Effect.fn("channelBalance")((path: bigint, token: Hex) =>
  pipe(
    ChannelDestination,
    Effect.andThen((config) =>
      readContract({
        address: config.ucs03address,
        abi: Ucs03.Abi,
        functionName: "_deprecated_channelBalanceV1",
        args: [config.channelId, path, token],
      })
    ),
  )
)

/**
 * @category utils
 * @since 0.0.0
 */
export const channelBalanceAtBlock = Effect.fn("channelBalanceAtBlock")(
  function*(path: bigint, token: Hex, blockNumber: bigint) {
    const client = (yield* PublicClientDestination).client
    const config = yield* ChannelDestination

    return yield* readContract({
      address: config.ucs03address,
      abi: Ucs03.Abi,
      functionName: "_deprecated_channelBalanceV1",
      args: [config.channelId, path, token],
      blockNumber: blockNumber,
    }).pipe(
      Effect.provideService(PublicClient, { client }),
    )
  },
)

/**
 * @category errors
 * @since 0.0.0
 */
export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError",
)<{
  cause: WaitForTransactionReceiptTimeoutErrorType
}> {}

/**
 * Wait for a transaction receipt
 * @param hash The transaction hash for which to wait
 * @returns An Effect that resolves to the transaction receipt
 *
 * @category utils
 * @since 0.0.0
 */
export const waitForTransactionReceipt = Effect.fn("waitForTransactionReceipt")((hash: Hash) =>
  pipe(
    PublicClient,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: () => client.waitForTransactionReceipt({ hash }),
        catch: err =>
          new WaitForTransactionReceiptError({
            cause: Utils.extractErrorDetails(err as WaitForTransactionReceiptTimeoutErrorType),
          }),
      })
    ),
  )
)

/**
 * A type-safe wrapper around viem's readContract that handles error cases
 * and returns an Effect with proper type inference. Extracts all error info
 *
 * @param client - The viem PublicClient to use for the contract call
 * @param params - The parameters for the contract call
 * @returns An Effect that resolves to the properly typed return value
 *
 * @category utils
 * @since 0.0.0
 */
export const readContract = Effect.fn("readContract")(<
  TAbi extends Abi,
  TFunctionName extends ContractFunctionName<TAbi, "pure" | "view"> = ContractFunctionName<
    TAbi,
    "pure" | "view"
  >,
  TArgs extends ContractFunctionArgs<TAbi, "pure" | "view", TFunctionName> = ContractFunctionArgs<
    TAbi,
    "pure" | "view",
    TFunctionName
  >,
>(
  params: ReadContractParameters<TAbi, TFunctionName, TArgs>,
) =>
  pipe(
    PublicClient,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: () => client.readContract(params),
        catch: error =>
          new ReadContractError({
            cause: Utils.extractErrorDetails(error as ReadContractErrorType),
          }),
      })
    ),
  )
)

/**
 * A type-safe wrapper around viem's writeContract that handles error cases
 * and returns an Effect with proper type inference. Extracts all error info
 *
 * @param client - The viem WalletClient to use for the contract transaction
 * @param params - The parameters for the contract transaction
 * @returns An Effect that resolves to the transaction hash
 *
 * @category utils
 * @since 0.0.0
 */
export const writeContract = Effect.fn("writeContract")(<
  const abi extends Abi | readonly unknown[],
  functionName extends ContractFunctionName<abi, "payable" | "nonpayable">,
  args extends ContractFunctionArgs<
    abi,
    "payable" | "nonpayable",
    functionName
  >,
  chainOverride extends ViemChain | undefined = undefined,
>(
  params: WriteContractParameters<abi, functionName, args>,
) =>
  pipe(
    WalletClient,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: () => client.writeContract(params),
        catch: error =>
          new WriteContractError({
            cause: Utils.extractErrorDetails(error as WriteContractErrorType),
          }),
      })
    ),
  )
)

/**
 * @category utils
 * @since 0.0.0
 */
export const simulateContract = <
  abi extends Abi | readonly unknown[] = Abi,
  functionName extends ContractFunctionName<
    abi,
    "nonpayable" | "payable"
  > = ContractFunctionName<abi, "nonpayable" | "payable">,
  args extends ContractFunctionArgs<
    abi,
    "nonpayable" | "payable",
    functionName
  > = ContractFunctionArgs<abi, "nonpayable" | "payable", functionName>,
  chain extends ViemChain | undefined = ViemChain | undefined,
  chainOverride extends ViemChain | undefined = ViemChain | undefined,
  accountOverride extends ViemAccount | Address | null | undefined = undefined,
  ///
  derivedChain extends ViemChain | undefined = DeriveChain<chain, chainOverride>,
>(
  params: SimulateContractParameters<
    abi,
    functionName,
    args,
    chain,
    chainOverride,
    accountOverride
  >,
) =>
  pipe(
    PublicClient,
    Effect.andThen(({ client }) =>
      Effect.tryPromise({
        try: () => client.simulateContract(params),
        catch: error =>
          new SimulateContractError({
            cause: Utils.extractErrorDetails(error as SimulateContractErrorType),
          }),
      })
    ),
  )

/**
 * @category context
 * @since 0.0.0
 */
export class ChannelDestination extends Context.Tag("@unionlabs/sdk/Evm/ChannelDestination")<
  ChannelDestination,
  Evm.Channel
>() {
  static Live = flow(
    ChannelDestination.of,
    Layer.succeed(this),
  )
}

/**
 * @category context
 * @since 0.0.0
 */
export class ChannelSource extends Context.Tag("@unionlabs/sdk/Evm/ChannelSource")<
  ChannelSource,
  Evm.Channel
>() {
  static Live = flow(
    ChannelSource.of,
    Layer.succeed(this),
  )
}

/**
 * @category context
 * @since 0.0.0
 */

export class PublicClientSource extends Context.Tag("@unionlabs/sdk/Evm/PublicClientSource")<
  PublicClientSource,
  Evm.PublicClient
>() {
  static Live = internal.publicClientLayer(this)
}

/**
 * @category context
 * @since 0.0.0
 */
export class PublicClientDestination
  extends Context.Tag("@unionlabs/sdk/Evm/PublicClientDestination")<
    PublicClientDestination,
    Evm.PublicClient
  >()
{
  static Live = internal.publicClientLayer(this)
}

/**
 * A neutral public client that can be used for general-purpose operations
 * that don't specifically target source or destination chains
 *
 * @category context
 * @since 0.0.0
 */
export class PublicClient extends Context.Tag("@unionlabs/sdk-evm/Evm/PublicClient")<
  PublicClient,
  Evm.PublicClient
>() {
  static Live = internal.publicClientLayer(this)
}

/**
 * A wallet client that can be used for signing transactions
 *
 * @category context
 * @since 0.0.0
 */
export class WalletClient extends Context.Tag("@unionlabs/sdk/Evm/WalletClient")<
  WalletClient,
  Evm.WalletClient
>() {
  static Live = internal.walletClientLayer(this)
}

/**
 * @category errors
 * @since 0.0.0
 */
export class ReadContractError extends Data.TaggedError("@unionlabs/sdk/Evm/ReadContractError")<{
  cause: ReadContractErrorType
}> {}

/**
 * @category errors
 * @since 0.0.0
 */
export class WriteContractError extends Data.TaggedError("@unionlabs/sdk/Evm/WriteContractError")<{
  cause: WriteContractErrorType
}> {}

/**
 * @category errors
 * @since 0.0.0
 */
export class SimulateContractError
  extends Data.TaggedError("@unionlabs/sdk/Evm/SimulateContractError")<{
    cause: SimulateContractErrorType
  }>
{}

/**
 * @category errors
 * @since 0.0.0
 */
export class CreatePublicClientError
  extends Data.TaggedError("@unionlabs/sdk/Evm/CreatePublicClientError")<{
    cause: CreatePublicClientErrorType
  }>
{}

/**
 * @category errors
 * @since 0.0.0
 */
export class CreateWalletClientError
  extends Data.TaggedError("@unionlabs/sdk/Evm/CreateWalletClientError")<{
    cause: CreateWalletClientErrorType
  }>
{}

/**
 * Read ERC20 token metadata (name, symbol, decimals)
 * @param tokenAddress The address of the ERC20 token
 * @param chainId The Universal chain ID to check for gas denomination
 * @returns An Effect that resolves to the token metadata
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20Meta = Effect.fn("readErc20Meta")(
  function*(tokenAddress: Address, chainId: UniversalChainId) {
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
    const name = yield* readErc20Name(tokenAddress)
    const symbol = yield* readErc20Symbol(tokenAddress)
    const decimals = yield* readErc20Decimals(tokenAddress)
    return { name, symbol, decimals }
  },
)

/**
 * Read the balance of an ERC20 token for a specific address
 *
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address to check the balance for
 *
 * @returns An Effect that resolves to the token balance
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20Balance = Effect.fn("readErc20Balance")((
  tokenAddress: Hex,
  ownerAddress: Ucs05.EvmDisplay,
) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "balanceOf",
    args: [ownerAddress.address],
  })
)

/**
 * Read the balance of an ERC20 token for a specific address
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address to check the balance for
 * @param blockNumber The blockNumber at certain point
 * @returns An Effect that resolves to the token balance
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20BalanceAtBlock = Effect.fn("readErc20BalanceAtBlock")((
  tokenAddress: Address,
  ownerAddress: Address,
  blockNumber: bigint,
) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "balanceOf",
    args: [ownerAddress],
    blockNumber: blockNumber,
  })
)

/**
 * Read the name of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token name
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20Name = Effect.fn("readErc20Name")((tokenAddress: Address) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "name",
  })
)

/**
 * Read the symbol of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token symbol
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20Symbol = Effect.fn("readErc20Symbol")((tokenAddress: Address) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "symbol",
  })
)

/**
 * Read the decimals of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the token decimals
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20Decimals = Effect.fn("readErc20Decimals")((tokenAddress: Address) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "decimals",
  })
)

/**
 * Read the TotalSupply of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @param blockNumber The blockNumber at certain point
 * @returns An Effect that resolves to the totalSupply
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20TotalSupplyAtBlock = Effect.fn("readErc20TotalSupplyAtBlock")((
  tokenAddress: Address,
  blockNumber: bigint,
) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "totalSupply",
    blockNumber: blockNumber,
  })
)

/**
 * Read the TotalSupply of an ERC20 token
 * @param tokenAddress The address of the ERC20 token
 * @returns An Effect that resolves to the totalSupply
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20TotalSupply = Effect.fn("readErc20TotalSupply")((tokenAddress: Address) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "totalSupply",
  })
)

/**
 * Read the allowance of an ERC20 token for a specific owner and spender
 * @param tokenAddress The address of the ERC20 token
 * @param ownerAddress The address of the token owner
 * @param spenderAddress The address of the spender
 * @returns An Effect that resolves to the token allowance
 *
 * @category utils
 * @since 0.0.0
 */
export const readErc20Allowance = Effect.fn("readErc20Allowance")((
  tokenAddress: Address,
  ownerAddress: Address,
  spenderAddress: Address,
) =>
  readContract({
    address: tokenAddress,
    abi: erc20Abi,
    functionName: "allowance",
    args: [ownerAddress, spenderAddress],
  })
)

/**
 * Increase the allowance of an ERC20 token for a specific spender
 * @param tokenAddress The address of the ERC20 token
 * @param spenderAddress The address of the spender
 * @param amount The amount to increase the allowance by
 * @returns An Effect that resolves to the transaction hash (on-chain hash for Safe wallets)
 *
 * @category utils
 * @since 0.0.0
 */
export const increaseErc20Allowance = Effect.fn("increaseErc20Allowance")((
  tokenAddress: Hex,
  spenderAddress: Ucs05.EvmDisplay,
  amount: bigint,
) =>
  pipe(
    WalletClient,
    Effect.andThen(({ client, account }) =>
      client.writeContract({
        account: account,
        chain: client.chain,
        address: tokenAddress,
        abi: erc20Abi,
        functionName: "approve",
        args: [spenderAddress.address, amount],
      })
    ),
    Effect.flatMap((txHash) =>
      pipe(
        Effect.serviceOption(Safe.Safe),
        Effect.flatMap(
          O.match({
            onNone: () => Effect.succeed(txHash),
            onSome: (safe) => safe.resolveTxHash(txHash),
          }),
        ),
      )
    ),
  )
)

/**
 * @category utils
 * @since 0.0.0
 */
export const sendInstruction = Effect.fn("sendInstruction")(
  function*(instruction: Ucs03.Instruction) {
    const walletClient = yield* WalletClient
    const sourceConfig = yield* ChannelSource

    const timeoutTimestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()
    const salt = yield* Utils.generateSalt("evm")

    const operand = yield* S.encode(Ucs03.InstructionFromHex)(instruction)

    return yield* writeContract({
      account: walletClient.account,
      abi: Ucs03.Abi,
      chain: walletClient.chain,
      functionName: "send",
      address: sourceConfig.ucs03address,
      args: [
        sourceConfig.channelId,
        0n,
        timeoutTimestamp,
        salt,
        {
          opcode: instruction.opcode,
          version: instruction.version,
          operand,
        },
      ],
      value: 10n,
    })
  },
)
