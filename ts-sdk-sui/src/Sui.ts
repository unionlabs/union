/**
 * This module handles EVM related functionality.
 *
 * @since 0.0.0
 */
import { GAS_DENOMS } from "@unionlabs/sdk/Constants"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { extractErrorDetails} from "@unionlabs/sdk/Utils"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Utils from "@unionlabs/sdk/Utils"
import { Context, Data, Effect, flow, Layer, pipe, Schema as S } from "effect"
import { type Address, erc20Abi } from "viem"
import type { Hex } from "viem"
import * as internal from "./internal/sui.js"
import { SuiClient, SuiClientOptions } from "@mysten/sui/client"
import { Transaction } from "@mysten/sui/transactions"

/**
 * @category models
 * @since 0.0.0
 */
export namespace Sui {
  /**
   * @category models
   * @since 0.0.0
   */
  export interface PublicClient {
    readonly client: SuiClient
  }

  /**
   * @category models
   * @since 0.0.0
   */
  export interface WalletClient {
    readonly client: SuiClient
    readonly signer: Ed25519Keypair
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

// /**
//  * @category utils
//  * @since 0.0.0
//  */
// export const channelBalance = (path: bigint, token: Hex) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClientDestination).client
//     const config = yield* ChannelDestination

//     const result = yield* readContract(client, {
//       address: config.ucs03address,
//       abi: Ucs03.Abi,
//       functionName: "_deprecated_channelBalanceV1",
//       args: [config.channelId, path, token],
//     })

//     return result
//   })

// /**
//  * @category utils
//  * @since 0.0.0
//  */
// export const channelBalanceAtBlock = (path: bigint, token: Hex, blockNumber: bigint) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClientDestination).client
//     const config = yield* ChannelDestination

//     const result = yield* readContract(client, {
//       address: config.ucs03address,
//       abi: Ucs03.Abi,
//       functionName: "_deprecated_channelBalanceV1",
//       args: [config.channelId, path, token],
//       blockNumber: blockNumber,
//     })

//     return result
//   })

/**
 * @category errors
 * @since 0.0.0
 */
export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError",
)<{
  cause: unknown
}> {}


export class ReadCoinError extends Data.TaggedError("ReadCoinError")<{
  cause: unknown
}> {}

// /**
//  * Wait for a transaction receipt
//  * @param hash The transaction hash for which to wait
//  * @returns An Effect that resolves to the transaction receipt
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const waitForTransactionReceipt = (hash: Hash) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     const receipt = yield* Effect.tryPromise({
//       try: () => client.waitForTransactionReceipt({ hash }),
//       catch: err =>
//         new WaitForTransactionReceiptError({
//           cause: Utils.extractErrorDetails(err as WaitForTransactionReceiptTimeoutErrorType),
//         }),
//     })

//     return receipt
//   })

export const readContract = <T>(
  client: SuiClient,
  sender: string,
  packageId: string,
  module: string,
  fn: string,
  typeArgs: string[],
  args: any[],
  tx: Transaction,
) =>
  Effect.tryPromise({
    try: async () => {
      tx.moveCall({
        target: `${packageId}::${module}::${fn}`,
        typeArguments: typeArgs,
        arguments: args,
      })
      const result = await client.devInspectTransactionBlock({
        transactionBlock: tx,
        sender,
      })
      return result.results // result as unknown as T
    },
    catch: e => new ReadContractError({ cause: extractErrorDetails(e as Error) }),
  }).pipe(
    // optional: e.g. timeout & retry like your Aptos wrapper
    Effect.timeout("10 seconds"),
    Effect.retry({ times: 5 }),
  )

export const writeContract = (
  client: SuiClient,
  signer: Ed25519Keypair,
  packageId: string,
  module: string,
  fn: string,
  typeArgs: string[],
  args: any[],
  tx: Transaction,
) =>
  Effect.tryPromise({
    try: async () => {
      tx.moveCall({
        target: `${packageId}::${module}::${fn}`,
        typeArguments: typeArgs,
        arguments: args,
      })
      // sign & execute
      const res = await client.signAndExecuteTransaction({
        signer,
        transaction: tx,
      })
      return res
    },
    catch: e => new WriteContractError({ cause: extractErrorDetails(e as Error) }),
  })

/**
 * @category context
 * @since 0.0.0
 */
export class ChannelDestination extends Context.Tag("@unionlabs/sdk/Sui/ChannelDestination")<
  ChannelDestination,
  Sui.Channel
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
export class ChannelSource extends Context.Tag("@unionlabs/sdk/Sui/ChannelSource")<
  ChannelSource,
  Sui.Channel
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

export class PublicClientSource extends Context.Tag("@unionlabs/sdk/Sui/PublicClientSource")<
  PublicClientSource,
  Sui.PublicClient
>() {
  static Live = internal.publicClientLayer(this)
}

/**
 * @category context
 * @since 0.0.0
 */
export class PublicClientDestination
  extends Context.Tag("@unionlabs/sdk/Sui/PublicClientDestination")<
    PublicClientDestination,
    Sui.PublicClient
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
export class PublicClient extends Context.Tag("@unionlabs/sdk-sui/Sui/PublicClient")<
  PublicClient,
  Sui.PublicClient
>() {
  static Live = internal.publicClientLayer(this)
}

/**
 * A wallet client that can be used for signing transactions
 *
 * @category context
 * @since 0.0.0
 */
export class WalletClient extends Context.Tag("@unionlabs/sdk/Sui/WalletClient")<
  WalletClient,
  Sui.WalletClient
>() {
  static Live = internal.walletClientLayer(this)
}

/**
 * @category errors
 * @since 0.0.0
 */
export class ReadContractError extends Data.TaggedError("@unionlabs/sdk/Sui/ReadContractError")<{
  cause: unknown
}> {}

/**
 * @category errors
 * @since 0.0.0
 */
export class WriteContractError extends Data.TaggedError("@unionlabs/sdk/Sui/WriteContractError")<{
  cause: unknown
}> {}

/**
 * @category errors
 * @since 0.0.0
 */
export class CreatePublicClientError
  extends Data.TaggedError("@unionlabs/sdk/Sui/CreatePublicClientError")<{
    cause: unknown
  }>
{}

/**
 * @category errors
 * @since 0.0.0
 */
export class CreateWalletClientError
  extends Data.TaggedError("@unionlabs/sdk/Sui/CreateWalletClientError")<{
    cause: unknown
  }>
{}

/**
 * Read Coin token metadata (name, symbol, decimals)
 * @param tokenAddress The address of the Coin token
 * @returns An Effect that resolves to the coin metadata
 *
 * @category utils
 * @since 0.0.0
 */
export const readCoinMetadata = (tokenAddress: Address) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const metadata = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getCoinMetadata({ coinType: tokenAddress })
        return result
      },
      catch: err =>
        new Error({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return metadata
  })

export const readCoinBalances = (contractAddress: string, address: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client
    let params = {
      owner: address,
      coinType: contractAddress,
    }

    const coins = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getCoins(params)
        return result.data
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return coins
  })

  export const readTotalCoinBalance = (contractAddress: string, address: string) =>
    Effect.gen(function*() {
      const client = (yield* PublicClient).client
      let params = {
        owner: address,
        coinType: contractAddress,
      }

      const coins = yield* Effect.tryPromise({
        try: async () => {
          const result = await client.getCoins(params)
          return result.data
        },
        catch: err =>
          new ReadCoinError({
            cause: extractErrorDetails(err as ReadCoinError),
          }),
      })
      // Calculate total balance
      const totalBalance = coins.reduce((acc, coin) => acc + BigInt(coin.balance), BigInt(0))

      return totalBalance
    })

  export const getAllCoins = (address: string) =>
    Effect.gen(function*() {
      const client = (yield* PublicClient).client
      let params = {
        owner: address,
      }

      const coins = yield* Effect.tryPromise({
        try: async () => {
          const result = await client.getAllCoins(params)
          return result.data
        },
        catch: err =>
          new ReadCoinError({
            cause: extractErrorDetails(err as ReadCoinError),
          }),
      })
      return coins
    })

  export const getAllCoinsUnique = (address: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const params = {
      owner: address,
    }

    const coins = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getAllCoins(params)
        return result.data
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })

    // Group by coinType and sum balances
    const coinMap: Record<string, bigint> = {}

    for (const coin of coins) {
      const coinType = coin.coinType
      const balance = BigInt(coin.balance)

      if (!coinMap[coinType]) {
        coinMap[coinType] = balance
      } else {
        coinMap[coinType] += balance
      }
    }

    // Convert to array of objects
    const result = Object.entries(coinMap).map(([coinType, totalBalance]) => ({
      coinType,
      balance: totalBalance.toString(), // or keep as BigInt if preferred
    }))

    return result
  })

  export const getCoinName = (address: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const name = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getCoinMetadata({ coinType: address })
        return result?.name
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return name
  })

  export const getCoinDecimals = (address: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const decimals = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getCoinMetadata({ coinType: address })
        return result?.decimals
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return decimals
  })

export const readCoinSymbol = (address: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const symbol = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getCoinMetadata({ coinType: address })
        return result?.symbol
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return symbol
  })


// /**
//  * Read the balance of an ERC20 token for a specific address
//  *
//  * @param tokenAddress The address of the ERC20 token
//  * @param ownerAddress The address to check the balance for
//  *
//  * @returns An Effect that resolves to the token balance
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20Balance = (tokenAddress: Address, ownerAddress: Address) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "balanceOf",
//       args: [ownerAddress],
//     })
//   })

// /**
//  * Read the balance of an ERC20 token for a specific address
//  * @param tokenAddress The address of the ERC20 token
//  * @param ownerAddress The address to check the balance for
//  * @param blockNumber The blockNumber at certain point
//  * @returns An Effect that resolves to the token balance
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20BalanceAtBlock = (
//   tokenAddress: Address,
//   ownerAddress: Address,
//   blockNumber: bigint,
// ) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "balanceOf",
//       args: [ownerAddress],
//       blockNumber: blockNumber,
//     })
//   })

// /**
//  * Read the name of an ERC20 token
//  * @param tokenAddress The address of the ERC20 token
//  * @returns An Effect that resolves to the token name
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20Name = (tokenAddress: Address) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "name",
//     })
//   })

// /**
//  * Read the symbol of an ERC20 token
//  * @param tokenAddress The address of the ERC20 token
//  * @returns An Effect that resolves to the token symbol
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20Symbol = (tokenAddress: Address) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "symbol",
//     })
//   })

// /**
//  * Read the decimals of an ERC20 token
//  * @param tokenAddress The address of the ERC20 token
//  * @returns An Effect that resolves to the token decimals
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20Decimals = (tokenAddress: Address) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "decimals",
//     })
//   })

// /**
//  * Read the TotalSupply of an ERC20 token
//  * @param tokenAddress The address of the ERC20 token
//  * @param blockNumber The blockNumber at certain point
//  * @returns An Effect that resolves to the totalSupply
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20TotalSupplyAtBlock = (tokenAddress: Address, blockNumber: bigint) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "totalSupply",
//       blockNumber: blockNumber,
//     })
//   })

// /**
//  * Read the TotalSupply of an ERC20 token
//  * @param tokenAddress The address of the ERC20 token
//  * @returns An Effect that resolves to the totalSupply
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20TotalSupply = (tokenAddress: Address) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "totalSupply",
//     })
//   })

// /**
//  * Read the allowance of an ERC20 token for a specific owner and spender
//  * @param tokenAddress The address of the ERC20 token
//  * @param ownerAddress The address of the token owner
//  * @param spenderAddress The address of the spender
//  * @returns An Effect that resolves to the token allowance
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const readErc20Allowance = (
//   tokenAddress: Address,
//   ownerAddress: Address,
//   spenderAddress: Address,
// ) =>
//   Effect.gen(function*() {
//     const client = (yield* PublicClient).client

//     return yield* readContract(client, {
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "allowance",
//       args: [ownerAddress, spenderAddress],
//     })
//   })

// /**
//  * Increase the allowance of an ERC20 token for a specific spender
//  * @param tokenAddress The address of the ERC20 token
//  * @param spenderAddress The address of the spender
//  * @param amount The amount to increase the allowance by
//  * @returns An Effect that resolves to the transaction hash
//  *
//  * @category utils
//  * @since 0.0.0
//  */
// export const increaseErc20Allowance = (
//   tokenAddress: Address,
//   spenderAddress: Address,
//   amount: bigint,
// ) =>
//   Effect.gen(function*() {
//     const walletClient = yield* WalletClient

//     return yield* writeContract({
//       account: walletClient.account,
//       chain: walletClient.chain,
//       address: tokenAddress,
//       abi: erc20Abi,
//       functionName: "approve",
//       args: [spenderAddress, amount],
//     })
//   })

// /**
//  * @category utils
//  * @since 0.0.0
//  */
// export const sendInstruction = (instruction: Ucs03.Instruction) =>
//   Effect.gen(function*() {
//     const walletClient = yield* WalletClient
//     const sourceConfig = yield* ChannelSource

//     const timeoutTimestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()
//     const salt = yield* Utils.generateSalt("evm")

//     const operand = yield* S.encode(Ucs03.InstructionFromHex)(instruction)

//     return yield* writeContract({
//       account: walletClient.account,
//       abi: Ucs03.Abi,
//       chain: walletClient.chain,
//       functionName: "send",
//       address: sourceConfig.ucs03address,
//       args: [
//         sourceConfig.channelId,
//         0n,
//         timeoutTimestamp,
//         salt,
//         {
//           opcode: instruction.opcode,
//           version: instruction.version,
//           operand,
//         },
//       ],
//       value: 10n,
//     })
//   })
