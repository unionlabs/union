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
      catch: err =>
        new Error({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
    })
    return metadata
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


  /**
 * PTB: begin_send -> (send_with_coin<T> ...)* -> end_send
 *
 * Mirrors:
 * sui client ptb \
 *   --move-call "$PKG::$MOD::begin_send" $CHANNEL_ID $SALT \
 *   --assign send_ctx1 \
 *   --move-call "$PKG::$MOD::send_with_coin" "<$TYPE_T>" \
 *       $RELAY_STORE $VAULT $IBC_STORE $COIN $VERSION $OPCODE $OPERAND send_ctx1 \
 *   --assign send_ctx2 \
 *   --move-call "$PKG::$MOD::end_send" $IBC_STORE $CLOCK $T_HEIGHT $T_TS send_ctx2 \
 *   --gas-budget 150000000
 */
export const sendInstruction = (params: {
  packageId: string

  /** coin type, e.g. "0x2::sui::SUI" (used in typeArguments of send_with_coin) */
  typeArg: string

  relayStoreId: string        // $RELAY_STORE
  vaultId: string             // $VAULT
  ibcStoreId: string          // $IBC_STORE
  coinObjectId: string        // $COIN

  // extraSendCalls?: Array<{
  //   relayStoreId?: string
  //   vaultId?: string
  //   ibcStoreId?: string
  //   coinObjectId?: string
  //   version?: number
  //   opcode?: number
  //   operandHex?: `0x${string}`
  //   typeArg?: string
  // }>

  /** the instruction used purely for encoding operand just like EVM */
  instruction: Ucs03.Instruction

}) =>
  Effect.gen(function* () {
    const module = "zkgm"
    const clockObjectId = "0x6" // Sui system clock object

    const { client, signer } = yield* WalletClient
    const channelId = (yield* ChannelSource).channelId

    const salt = yield* Utils.generateSalt("evm") // TODO: check if evm will work here or not
    const timeoutNs = Utils.getTimeoutInNanoseconds24HoursFromNow()
    const tHeight = BigInt(0)

    const operandHex = (yield* S.encode(Ucs03.InstructionFromHex)(params.instruction)) as `0x${string}`

    // helpers
    const hexToBytes = (hex: `0x${string}`): Uint8Array => {
      const s = hex.slice(2)
      const out = new Uint8Array(s.length / 2)
      for (let i = 0; i < out.length; i++) out[i] = parseInt(s.slice(i * 2, i * 2 + 2), 16)
      return out
    }

    const tx = new Transaction()
    // if (params.gasBudget !== undefined) tx.setGasBudget(BigInt(params.gasBudget as any))

    let sendCtx = tx.moveCall({
      target: `${params.packageId}::${module}::begin_send`,
      typeArguments: [],
      arguments: [
        tx.pure.u32(channelId), 
        tx.pure.vector("u8", hexToBytes(salt as `0x${string}`)), 
      ],
    })

    const pushSendWithCoin = (cfg: {
      relayStoreId: string
      vaultId: string
      ibcStoreId: string
      coinObjectId: string
      version: number
      opcode: number
      operandHex: `0x${string}`
      typeArg: string
    }) => {
      sendCtx = tx.moveCall({
        target: `${params.packageId}::${module}::send_with_coin`,
        typeArguments: [cfg.typeArg],
        arguments: [
          tx.object(cfg.relayStoreId),
          tx.object(cfg.vaultId),
          tx.object(cfg.ibcStoreId),
          tx.object(cfg.coinObjectId),
          tx.pure.u8(cfg.version),
          tx.pure.u8(cfg.opcode), 
          tx.pure.vector("u8", hexToBytes(cfg.operandHex)),
          sendCtx,
        ],
      })
    }

    pushSendWithCoin({
      relayStoreId: params.relayStoreId,
      vaultId: params.vaultId,
      ibcStoreId: params.ibcStoreId,
      coinObjectId: params.coinObjectId,
      version: params.instruction.version,
      opcode: params.instruction.opcode,
      operandHex,
      typeArg: params.typeArg,
    })

    // TODO: multiple send_with_coin calls if needed??? will this work?
    // for (const extra of params.extraSendCalls ?? []) {
    //   pushSendWithCoin({
    //     relayStoreId: extra.relayStoreId ?? params.relayStoreId,
    //     vaultId: extra.vaultId ?? params.vaultId,
    //     ibcStoreId: extra.ibcStoreId ?? params.ibcStoreId,
    //     coinObjectId: extra.coinObjectId ?? params.coinObjectId,
    //     version: extra.version ?? params.version,
    //     opcode: extra.opcode ?? params.opcode,
    //     operandHex: (extra.operandHex ?? operandHex) as `0x${string}`,
    //     typeArg: extra.typeArg ?? params.typeArg,
    //   })
    // }

    tx.moveCall({
      target: `${params.packageId}::${module}::end_send`,
      typeArguments: [],
      arguments: [
        tx.object(params.ibcStoreId),
        tx.object(clockObjectId),
        tx.pure.u64(tHeight),
        tx.pure.u64(BigInt(timeoutNs)), // ns
        sendCtx,
      ],
    })

    const res = yield* Effect.tryPromise({
      try: async () =>
        client.signAndExecuteTransaction({
          signer,
          transaction: tx,
        }),
      catch: (e) => new WriteContractError({ cause: extractErrorDetails(e as Error) }),
    })

    return res
  })
  
// turn a hex string like "0xdeadbeef" into a number[] of bytes
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16))
}

// // TODO: Decide the parameters here.
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
//       client: walletClient.client,
//       signer: walletClient.signer,
//       account: walletClient.account,
//       abi: Ucs03.Abi, 
//       // chain: walletClient.chain, TODO: Do we need this?
//       fn: "send",
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
