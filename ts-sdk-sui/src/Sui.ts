/**
 * This module handles Sui related functionality.
 *
 * @since 0.0.0
 */
import { SuiClient } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"
import { extractErrorDetails } from "@unionlabs/sdk/Utils"
import { Context, Data, Effect, flow, Layer } from "effect"
import { type Address } from "viem"
import * as internal from "./internal/sui.js"

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
    readonly rpc: string
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

export class ReadCoinError extends Data.TaggedError("ReadCoinError")<{
  cause: unknown
}> {}

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
 * Read Coin metadata (name, symbol, decimals, …) for a given `coinType`.
 *
 * Example:
 * ```ts
 * const meta = yield* readCoinMetadata("0x2::sui::SUI")
 * ```
 *
 * @param tokenAddress Canonical coin type string (e.g., `"0x2::sui::SUI"`)
 * @returns Effect resolving to `getCoinMetadata` result
 * @throws ReadCoinError on RPC failure
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
      catch: cause => new ReadCoinError({ cause }),
    })
    return metadata
  })

/**
 * Read Sui coin metadata (name, symbol, decimals) for a given `coinType`.
 *
 * Example:
 * ```ts
 * const meta = yield* readCoinMeta("0x2::sui::SUI")
 * // -> { name: "Sui", symbol: "SUI", decimals: 9 }
 * ```
 *
 * @param coinType Canonical coin type string (e.g., "0x2::sui::SUI")
 * @returns Effect resolving to `{ name, symbol, decimals }`
 * @throws ReadCoinError on RPC failure
 *
 * @category utils
 * @since 0.0.0
 */
export const readCoinMeta = (coinType: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const out = yield* Effect.tryPromise({
      try: async () => {
        const meta = await client.getCoinMetadata({ coinType })
        // meta can be null if the type has no metadata published
        if (!meta) {
          // normalize to a typed error consistent with your pattern
          throw new ReadCoinError({ cause: `No CoinMetadata found for ${coinType}` })
        }
        const { name, symbol, decimals } = meta
        return { name, symbol, decimals }
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as Error),
        }),
    })

    return out
  })

/**
 * Read all coin objects for a given `coinType` and owner address.
 *
 * Note:
 * - Sui splits balances across multiple coin objects; each carries `balance` and `coinObjectId`.
 * - Use {@link readTotalCoinBalance} if you want a single summed value.
 *
 * @param contractAddress Canonical coin type (e.g., `"0x2::sui::SUI"`)
 * @param address         Owner Sui address
 * @returns Effect resolving to the paged coin objects’ `data` array
 *
 * @category utils
 * @since 0.0.0
 */
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

/**
 * Resolve the signer address from WalletClient.
 */
export const getSignerAddress = Effect.gen(function*() {
  const { signer } = yield* WalletClient
  return signer.toSuiAddress()
})

/**
 * Incrementally fetch coin objects for `owner` and `coinType` until the running
 */
export const getCoinsWithBalance = (coinType: string, min: bigint) =>
  Effect.gen(function*() {
    const { client } = yield* PublicClient
    const resolvedOwner = yield* getSignerAddress

    return yield* Effect.tryPromise({
      try: async () => {
        let cursor: string | null | undefined = undefined
        let acc: Array<{ coinObjectId: string; balance: string }> = []
        let total = 0n

        while (true) {
          const page = await client.getCoins({ owner: resolvedOwner, coinType: coinType, cursor, limit: 50 })
          for (const c of page.data) {
            acc.push({ coinObjectId: c.coinObjectId, balance: c.balance })
            total += BigInt(c.balance)
            if (total >= min) {
              return { coins: acc, total, hasEnough: true as const }
            }
          }
          if (!page.hasNextPage) {
            break
          }
          cursor = page.nextCursor
        }

        return { coins: acc, total, hasEnough: false as const }
      },
      catch: (err) => new ReadCoinError({ cause: extractErrorDetails(err as Error) }),
    })
  })

/**
 * Prepare a coin for spending inside the SAME PTB:
 */
export const prepareCoinForAmount = (
  tx: Transaction,
  coinType: string,
  amount: bigint,
  owner: string,
): Effect.Effect<TransactionObjectArgument, ReadCoinError, PublicClient | WalletClient> =>
  Effect.gen(function*() {
    // SUI special case: split from gas
    if (
      coinType === "0x2::sui::SUI"
      || coinType === "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"
    ) {
      const [out] = tx.splitCoins(tx.gas, [tx.pure.u64(amount)])
      return out
    }
    const { coins, hasEnough } = yield* getCoinsWithBalance(coinType, amount)
    if (!hasEnough || coins.length === 0) {
      return yield* Effect.fail(
        new ReadCoinError({ cause: `Insufficient ${coinType} balance for split ${amount}` }),
      )
    }

    const target = coins[0]
    const targetArg = tx.object(target.coinObjectId)
    const rest = coins.slice(1).map(c => tx.object(c.coinObjectId))
    if (rest.length > 0) {
      tx.mergeCoins(targetArg, rest)
    }

    const [out] = tx.splitCoins(targetArg, [tx.pure.u64(amount)])
    return out
  })

/**
 * Read and sum all coin object balances for a given `coinType` and owner.
 *
 * @param contractAddress Canonical coin type
 * @param address         Owner Sui address
 * @returns Effect resolving to a `bigint` total (in the coin’s base units)
 *
 * @category utils
 * @since 0.0.0
 */
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

/**
 * Fetch *all* coin objects (any coin type) for an owner.
 *
 * @param address Owner Sui address
 * @returns Effect resolving to `getAllCoins().data`
 *
 * @category utils
 * @since 0.0.0
 */
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

/**
 * Fetch all coins for an owner and return a unique list grouped by `coinType`,
 * with balances summed across coin objects.
 *
 * Example output:
 * ```ts
 * [
 *   { coinType: "0x2::sui::SUI", balance: "123456789" },
 *   { coinType: "0x...::USDC::USDC", balance: "4200000" }
 * ]
 * ```
 *
 * @param address Owner Sui address
 * @returns Effect resolving to `{ coinType, balance }[]` (balance as string)
 *
 * @category utils
 * @since 0.0.0
 */
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

/**
 * Convenience: read coin **name** for a given `coinType`.
 *
 * @param address Canonical coin type
 * @returns Effect resolving to `string | undefined`
 *
 * @category utils
 * @since 0.0.0
 */
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

/**
 * Convenience: read coin **decimals** for a given `coinType`.
 *
 * @param address Canonical coin type
 * @returns Effect resolving to `number | undefined`
 *
 * @category utils
 * @since 0.0.0
 */
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

/**
 * Convenience: read coin **symbol** for a given `coinType`.
 *
 * @param address Canonical coin type
 * @returns Effect resolving to `string | undefined`
 *
 * @category utils
 * @since 0.0.0
 */
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
