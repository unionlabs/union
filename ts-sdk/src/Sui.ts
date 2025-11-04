/**
 * This module handles [Sui](https://sui.io/) related functionality.
 *
 * :::caution[INCOMPLETE]
 * This module is incomplete. Functionality may be partial. Breaking changes may be necessary and regular.
 * :::
 *
 * @since 2.0.0
 */
import { bcs } from "@mysten/sui/bcs"
import { getFullnodeUrl, SuiClient } from "@mysten/sui/client"
import type { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"
import { Context, Data, Effect, flow, Layer } from "effect"
import type { Address } from "viem"
import type { Hex } from "viem"
import * as internal from "./internal/sui.js"
import { extractErrorDetails } from "./utils/extract-error-details.js"

/**
 * @category models
 * @since 2.0.0
 */
export namespace Sui {
  /**
   * @category models
   * @since 2.0.0
   */
  export interface PublicClient {
    readonly client: SuiClient
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export interface WalletClient {
    readonly client: SuiClient
    readonly signer: Ed25519Keypair
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export interface Channel {
    readonly ucs03address: Address
    readonly channelId: number
  }
}

/**
 * @category context
 * @since 2.0.0
 */
export class FungibleAssetOrderDetails
  extends Context.Tag("@unionlabs/sdk/Sui/FungibleAssetOrderDetails")<
    FungibleAssetOrderDetails,
    {
      readonly typename_t: Hex
      readonly ibc_store: Hex
      readonly relay_store: Hex
      readonly coin: Hex
      readonly metadata: Hex
    }
  >()
{}

/**
 * @category context
 * @since 2.0.0
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
 * @since 2.0.0
 */
export class ChannelSource extends Context.Tag("@unionlabs/sdk/Sui/ChannelSource")<
  ChannelSource,
  Sui.Channel
>() {
  static Live = flow(
    ChannelDestination.of,
    Layer.succeed(this),
  )
}

/**
 * @category context
 * @since 2.0.0
 */
export class PublicClient extends Context.Tag("@unionlabs/sdk/Sui/PublicClient")<
  PublicClient,
  Sui.PublicClient
>() {
  static Live = internal.publicClientLayer(this)
  static FromNode = (url: Parameters<typeof getFullnodeUrl>[0]) =>
    internal.publicClientLayer(this)({
      url: getFullnodeUrl(url),
    })
}

/**
 * @category context
 * @since 2.0.0
 */
export class PublicClientSource extends Context.Tag("@unionlabs/sdk/Sui/PublicClientSource")<
  PublicClientSource,
  Sui.PublicClient
>() {
  static Live = internal.publicClientLayer(this)
  static FromNode = (url: Parameters<typeof getFullnodeUrl>[0]) =>
    internal.publicClientLayer(this)({
      url: getFullnodeUrl(url),
    })
}

/**
 * @category context
 * @since 2.0.0
 */
export class PublicClientDestination
  extends Context.Tag("@unionlabs/sdk/Sui/PublicClientDestination")<
    PublicClientDestination,
    Sui.PublicClient
  >()
{
  static Live = internal.publicClientLayer(this)
  static FromNode = (url: Parameters<typeof getFullnodeUrl>[0]) =>
    internal.publicClientLayer(this)({
      url: getFullnodeUrl(url),
    })
}

/**
 * A wallet client that can be used for signing transactions
 *
 * @category context
 * @since 2.0.0
 */
export class WalletClient extends Context.Tag("@unionlabs/sdk/Sui/WalletClient")<
  WalletClient,
  {
    readonly client: SuiClient
    readonly signer: Ed25519Keypair
  }
>() {}

/**
 * Interface for FA token metadata
 *
 * @category models
 * @since 2.0.0
 */
export interface FaTokenInfo {
  decimals: number
  icon_uri: string
  name: string
  project_uri: string
  symbol: string
}

/**
 * @category errors
 * @since 2.0.0
 */
export class ReadCoinError extends Data.TaggedError("@unionlabs/sdk/Sui/ReadCoinError")<{
  cause: unknown
}> {}

/**
 * @category errors
 * @since 2.0.0
 */
export class ReadContractError extends Data.TaggedError("@unionlabs/sdk/Sui/ReadContractError")<{
  cause: unknown
}> {}

/**
 * @category errors
 * @since 2.0.0
 */
export class WriteContractError extends Data.TaggedError("@unionlabs/sdk/Sui/WriteContractError")<{
  cause: unknown
}> {}

/**
 * @category errors
 * @since 2.0.0
 */
export class CreateWalletClientErrorType
  extends Data.TaggedError("@unionlabs/sdk/Sui/CreateWalletClientErrorType")<{
    cause: unknown
  }>
{}

/**
 * @category errors
 * @since 2.0.0
 */
export class CreatePublicClientErrorType
  extends Data.TaggedError("@unionlabs/sdk/Sui/CreatePublicClientErrorType")<{
    cause: unknown
  }>
{}

/**
 * @category errors
 * @since 2.0.0
 */
export class CreatePublicClientError
  extends Data.TaggedError("@unionlabs/sdk/Sui/CreatePublicClientError")<{
    cause: CreatePublicClientErrorType
  }>
{}

/**
 * @category errors
 * @since 2.0.0
 */
export class CreateWalletClientError
  extends Data.TaggedError("@unionlabs/sdk/Sui/CreateWalletClientError")<{
    cause: CreateWalletClientErrorType
  }>
{}

/**
 * @category utils
 * @since 2.0.0
 */
export const channelBalance = (path: number, token: Hex, relayStore: Hex) =>
  Effect.gen(function*() {
    const client = (yield* PublicClientDestination).client
    const config = yield* ChannelDestination

    const contract_address = config.ucs03address
    const module_id = "zkgm_relay"
    const function_name = "channel_balance"

    const tx = new Transaction()
    const function_arguments = [
      tx.object(relayStore),
      tx.pure.u32(config.channelId),
      tx.pure.u256(path),
      tx.pure("vector<u8>", hexToBytes(token)),
    ]

    yield* Effect.log("Getting channel_balance for token:", token)

    const result = yield* readContract(
      client,
      "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
      contract_address,
      module_id,
      function_name,
      [],
      function_arguments,
      tx,
    )
    if (!result || result.length === 0 || !result[0].returnValues || !result[0].returnValues[0]) {
      throw new Error("No return value from channel_balance")
    }
    const [bytesArray] = result[0].returnValues[0] as [number[], string]
    const data = new Uint8Array(bytesArray)
    const decoded = bcs.U256.parse(data)

    return decoded
  })

/**
 * Turn a hex string like "0xdeadbeef" into a number[] of bytes.
 *
 * @category utils
 * @since 2.0.0
 */
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16))
}

/**
 * @category utils
 * @since 2.0.0
 */
function bytesToHex(bytes: number[]) {
  return "0x" + bytes.map(b => b.toString(16).padStart(2, "0")).join("")
}

/**
 * @category utils
 * @since 2.0.0
 */
export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function*() {
    const client = (yield* PublicClientDestination).client
    const config = yield* ChannelDestination
    yield* Effect.log(
      `Predicting quote token for base token: ${baseToken} at channel: ${config.channelId} on ZKGM Address: ${config.ucs03address}`,
    )

    const contract_address = config.ucs03address
    const module_id = "zkgm_relay"
    const function_name = "compute_salt"
    const converted_base_token = baseToken

    const tx = new Transaction()
    const function_arguments = [
      tx.pure.u256(0),
      tx.pure.u32(config.channelId),
      tx.pure("vector<u8>", hexToBytes(converted_base_token)),
    ]

    const result = yield* readContract(
      client,
      "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
      contract_address,
      module_id,
      function_name,
      [],
      function_arguments,
      tx,
    )

    if (!result || result.length === 0 || !result[0].returnValues || !result[0].returnValues[0]) {
      throw new Error("No return value from compute_salt")
    }
    const [rawBytes /*, _typeTag*/] = result[0].returnValues[0] as [number[], string]

    return bytesToHex(rawBytes.slice(1))
  })

/**
 * @category utils
 * @since 2.0.0
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
 * @category utils
 * @since 2.0.0
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
 * @category utils
 * @since 2.0.0
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
 * @category utils
 * @since 2.0.0
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
 * @category utils
 * @since 2.0.0
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
 * @category utils
 * @since 2.0.0
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
 * @category utils
 * @since 2.0.0
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

/**
 * @category utils
 * @since 2.0.0
 */
export const readCoinMetadata = (address: string) =>
  Effect.gen(function*() {
    const client = (yield* PublicClient).client

    const metadata = yield* Effect.tryPromise({
      try: async () => {
        const result = await client.getCoinMetadata({ coinType: address })
        return result
      },
      catch: err =>
        new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return metadata
  })

/**
 * @category utils
 * @since 2.0.0
 */
export const readContract = (
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
      // dev-inspect it
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

/**
 * @category utils
 * @since 2.0.0
 */
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
