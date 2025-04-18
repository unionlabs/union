import { Effect, Schedule, Data, Context, Logger } from "effect"
import { createPublicClient, http } from "viem"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import type { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate"
import { GasPrice } from "@cosmjs/stargate"
import { coins } from "@cosmjs/proto-signing"

import {
  channelBalance as EthereumChannelBalance,
  readErc20TotalSupply,
  ViemPublicClient as ViemPublicClientContext,
  ViemPublicClientDestination,
  EvmChannelDestination,
  readErc20Balance
} from "@unionlabs/sdk/evm"

import {
  channelBalance as CosmosChannelBalance,
  readCw20TotalSupply,
  createCosmWasmClient,
  CosmWasmClientContext,
  CosmWasmClientDestination,
  CosmosChannelDestination,
  createSigningCosmWasmClient
} from "@unionlabs/sdk/cosmos"

import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import fs from "node:fs"
import type { Address } from "viem"
import { request, gql } from "graphql-request"
import { fromHex } from "viem"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

type Hex = `0x${string}`

function hexToUtf8(hex: string): string {
  // strip optional 0x
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  // build a Buffer from hex, then decode as UTF‑8
  return Buffer.from(clean, "hex").toString("utf8");
}

// Chain pair configuration
interface ChainPair {
  sourceChain: string
  destinationChain: string
  timeframeMs: number
  enabled: boolean
}

// EVM transfer configuration
interface TransferConfig {
  enabled: boolean
  privateKey: string
  sourceChainIdEVM: string
  sourceChainIdCosmos: string // TODO: Change them later
  destinationChainId: string // TODO: Change them later
  sourceChainIdAptos: string // TODO: Change them later
  rpcs: Array<string>
  gasPriceDenom: string
  receiverAddress: Address
  denomAddress: Address
  amount_range: Array<bigint>
  cosmosAccountType: string
}

interface WrappedToken {
  chain: { universal_chain_id: string }
  denom: Hex
  wrapping: Array<{
    unwrapped_chain: { universal_chain_id: string }
    destination_channel_id: number
    unwrapped_denom: string
  }>
}

interface FundableAccounts {
  receiver_display: string
  traces: Array<{
    type: string
    transaction_hash: string
  }>
}

interface V2Channels {
  source_channel_id: string 
}

interface ChannelInfo {
  source_channel_id: number
}

interface Packet {
  packet_send_timestamp: string | null
  packet_recv_timestamp: string | null
  write_ack_timestamp: string | null
  packet_ack_timestamp: string | null
  packet_send_transaction_hash?: string | null
  packet_recv_transaction_hash?: string | null
  write_ack_transaction_hash?: string | null
  packet_ack_transaction_hash?: string | null
  sort_order: string
  packet_send_block_hash: string
  packet_hash: string
  timeout_timestamp: string
}

interface HasuraResponse {
  v1_ibc_union_packets: Packet[]
}

type ChainType = "evm" | "cosmos"

interface ChainConfigEntry {
  zkgmAddress: string
  rpc: string
  chainType: ChainType
  minter: string
}

type ChainConfig = Record<string, ChainConfigEntry>

// Combined configuration shape
interface ConfigFile {
  interactions: Array<ChainPair>
  cycleIntervalMs: number
  transfers?: Array<TransferConfig>
  hasuraEndpoint: string
  chainConfig: ChainConfig
  signer_account_mnemonic: string
}

// A module-level set to keep track of already reported packet send transaction hashes.
// Variable to track sleep cycles
let sleepCycleCount = 0

// export class Transfer extends Schema.Class<Transfer>("Transfer")({
//   token: Schema.Literal("0x09B8aE6BB8D447bF910068E6c246A270F42b41be", "0x09B8aE6BB8D447bF910068E6c246A270F42b41be"),
//   amount: Schema.Int.pipe(Schema.between(1, 80)),
//   hex: Schema.String.pipe(Schema.pattern(/^0x[0-9a-fA-F]+$/))
// }) {}

class DoTransferError extends Data.TaggedError("DoTransferError")<{
  cause: unknown
}> {}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

export class Config extends Context.Tag("Config")<Config, { readonly config: ConfigFile }>() {}

const doTransferRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(2)) // Limit retries to 2
)

const fetchWrappedTokens = (hasuraEndpoint: string) =>
  Effect.gen(function* () {
    const query = gql`
    query WrappedTokens {
      v2_tokens(where: { wrapping: { unwrapped_denom: { _is_null: false } } }) {
        chain { universal_chain_id }
        denom
        wrapping {
          unwrapped_chain { universal_chain_id }
          destination_channel_id
          unwrapped_denom
        }
      }
    }
  `

    const response: any = yield* Effect.tryPromise({
      try: () => request(hasuraEndpoint, query),
      catch: error => {
        console.error("fetchWrappedTokens failed:", error)
        throw error
      }
    })

    const tokens: WrappedToken[] = response?.v2_tokens || []
    return tokens
  })


  const fetchFundableAccounts = (hasuraEndpoint: string) =>
    Effect.gen(function* () {
      const query = gql`
      query {
        v2_transfers(args: { p_destination_universal_chain_id: "babylon.bbn-1" }) {
          receiver_display
          traces {
            type
            transaction_hash
          }
        }
      }
    `

      const response: any = yield* Effect.tryPromise({
        try: () => request(hasuraEndpoint, query),
        catch: error => {
          console.error("fetchFundableAccounts failed:", error)
          throw error
        }
      })
  
      const tokens: FundableAccounts[] = response?.v2_transfers || []
      const filtered: FundableAccounts[] = tokens
      .map(({ receiver_display, traces }) => ({
        receiver_display,
        traces: traces
          .filter(trace => trace.type === "WRITE_ACK" && trace.transaction_hash != null)
          .map(trace => ({ type: trace.type, transaction_hash: trace.transaction_hash! }))
      }))
      // remove any where we ended up with zero traces
      .filter(acc => acc.traces.length > 0)

    return filtered
  })

  const fetchChannelsViaChainId = (hasuraEndpoint: string, chainId: string) =>
    Effect.gen(function* () {
      const query = gql`
      query GetChannels($sourceUniversalChainId: String!) {
        v2_channels(
          where: { source_universal_chain_id: { _eq: $sourceUniversalChainId } }
        ) {
          source_channel_id
        }
      }
    `
  
      const response: any = yield* Effect.tryPromise({
        try: () => request(hasuraEndpoint, query, { sourceUniversalChainId: chainId }),
        catch: error => {
          console.error("fetchWrappedTokens failed:", error)
          throw error
        }
      })
  
      const channels: V2Channels[] = response?.v2_channels || []
      return channels
    })

const fetchSourceChannelId = (
  hasuraEndpoint: string,
  srcChain: string,
  dstChain: string,
  dstChannelId: number
) =>
  Effect.gen(function* () {
    const query = gql`
    query ChannelInfo($src: String!, $dst: String!, $dchan: Int!) {
      v2_channels(args: {
        p_source_universal_chain_id: $src,
        p_destination_universal_chain_id: $dst,
        p_destination_channel_id: $dchan
      }) {
        source_channel_id
      }
    }
  `

    const response: any = yield* Effect.tryPromise({
      try: () =>
        request(hasuraEndpoint, query, { src: srcChain, dst: dstChain, dchan: dstChannelId }),
      catch: error => {
        console.error("fetchSourceChannelId failed:", error)
        throw error
      }
    })

    const channels: ChannelInfo[] = response?.v2_channels || []
    return channels[0]?.source_channel_id
  })

function loadConfig(configPath: string) {
  return Effect.tryPromise({
    try: async () => {
      if (!fs.existsSync(configPath)) {
        throw new Error("Config file not found. Ensure config.json exists.")
      }
      const rawData = fs.readFileSync(configPath, "utf-8")
      const config: ConfigFile = JSON.parse(rawData)
      if (!Array.isArray(config.interactions) || config.interactions.length === 0) {
        throw new Error("Config file is invalid or interactions array is empty.")
      }
      return config
    },
    catch: error =>
      new FilesystemError({
        message: "Config file is invalid or interactions array is empty.",
        cause: error
      })
  })
}

// const createBatchFromTransfers = (
//   transfers: readonly {
//     sender: Hex
//     receiver: Address
//     baseToken: Hex
//     baseAmount: bigint
//     quoteAmount: bigint
//   }[]
// ) =>
//   Effect.gen(function* () {
//     const transferInstructions = []
//     for (const transfer of transfers) {
//       yield* Effect.log(`creating transfer for ${transfer.baseToken}`)
//       const transferInstruction = yield* createEvmToCosmosFungibleAssetOrder(transfer)
//       transferInstructions.push(transferInstruction)
//     }
//     return Batch(transferInstructions)
//   }).pipe(Effect.withLogSpan("batch creation"))

// const doTransfer = (task: TransferConfig) =>
//   Effect.gen(function* (_) {
//     let chainType: "Cosmos" | "EVM" | "Aptos"
//     let sourceChainId: string

//     if (task.sourceChainIdCosmos) {
//       chainType = "Cosmos"
//       sourceChainId = task.sourceChainIdCosmos
//     } else if (task.sourceChainIdAptos) {
//       chainType = "Aptos"
//       sourceChainId = task.sourceChainIdAptos
//     } else {
//       chainType = "EVM"
//       sourceChainId = task.sourceChainIdEVM
//     }

//     const arb = Arbitrary.make(Schema.Int.pipe(Schema.between(1, 80)))
//     const random_amount = BigInt(FastCheck.sample(arb, 1)[0]!)

//     yield* Effect.log(
//       `\n[${chainType}] Starting transfer for chainId=${sourceChainId} to chain=${task.destinationChainId}`
//     )

//     const account = privateKeyToAccount(`0x${task.privateKey.replace(/^0x/, "")}`)
//     const tokenAddress = task.denomAddress
//     const receiver = task.receiverAddress

//     const publicSourceClient = yield* createViemPublicClient({
//       chain: sepolia,
//       transport: http()
//     })

//     // const walletClient = createWalletClient({
//     //   account,
//     //   chain: sepolia,
//     //   transport: http()
//     // })

//     // const tx_hash = yield* writeContract(walletClient, {
//     //   account,
//     //   chain: sepolia,
//     //   address: tokenAddress,
//     //   abi: erc20Abi,
//     //   functionName: "transfer",
//     //   args: [task.receiverAddress, random_amount]
//     // }).pipe(
//     //   Effect.provideService(ViemWalletClient, { client: walletClient }),
//     //   Effect.mapError(e => e.cause.message)
//     // )

//     // yield* Effect.log("Transfer tx hash:", tx_hash)

//     // Define hardcoded UCS03 addresses and channel ids for now
//     const UCS03_ADDRESS = "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962" // UCS03 contract on Sepolia
//     const CHANNEL_ID = 1 // Hardcoded channel ID

//     const TRANSFERS = [
//       {
//         sender: account.address,
//         receiver: receiver,
//         baseToken: tokenAddress, // Token to transfer
//         baseAmount: random_amount,
//         quoteAmount: 0n // Quote amount is not used in this case
//       }
//     ] as const

//     yield* Effect.log("Attempting to create CosmWasm client...")
//     const cosmWasmClientDestination = yield* createCosmWasmClient(
//       "https://rpc.rpc-node.union-testnet-10.union.build"
//     ).pipe(
//       Effect.catchTag("CosmWasmClientError", error =>
//         Effect.gen(function* () {
//           yield* Effect.logError("CosmWasm client creation failed:", error)
//           return yield* Effect.fail(new DoTransferError({ cause: error }))
//         })
//       )
//     )

//     yield* Effect.log("CosmWasm client created successfully")

//     yield* Effect.log("Transfers to be made:", TRANSFERS)

//     yield* Effect.gen(function* () {
//       const batch = yield* createBatchFromTransfers(TRANSFERS)
//       yield* Effect.log("Batch created:", batch)
//     }).pipe(
//       Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
//       Effect.provideService(ViemPublicClientSource, { client: publicSourceClient }),
//       Effect.provideService(CosmosChannelDestination, {
//         ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
//         channelId: 1
//       })
//       // Effect.provideService(EvmChannelSource, {
//       //   ucs03address: UCS03_ADDRESS,
//       //   channelId: 1
//       // }),
//       // Effect.provideService(ViemWalletClient, {
//       //   client: walletClient,
//       //   account: account,
//       //   chain: sepolia
//       // })
//     )
//     // const createBatch = Effect.gen(function* () {
//     //   const transferInstructions = []

//     //   for (const transfer of TRANSFERS) {
//     //     yield* Effect.log(`creating transfer for ${transfer.baseToken}`)
//     //     const transferInstruction = yield* createEvmToCosmosFungibleAssetOrder(transfer)
//     //     transferInstructions.push(transferInstruction)
//     //   }

//     //   return Batch(transferInstructions)
//     // }).pipe(Effect.withLogSpan("batch creation"))

//     // yield* Effect.log("Creating batch...")
//     // const batch = yield* createBatch
//     // yield* Effect.log("Batch created:", batch)

//     // const checkAndIncreaseAllowances = Effect.gen(function* () {
//     //   yield* Effect.log("Checking token allowances...")

//     //   for (const transfer of TRANSFERS) {
//     //     yield* Effect.log(`checking ${transfer.baseToken} allowance...`)

//     //     // Check current allowance
//     //     const currentAllowance = yield* readErc20Allowance(
//     //       transfer.baseToken,
//     //       transfer.sender,
//     //       UCS03_ADDRESS
//     //     )

//     //     yield* Effect.log(`current ${transfer.baseToken} allowance: ${currentAllowance}`)

//     //     // If allowance is insufficient, increase it
//     //     if (currentAllowance < transfer.baseAmount) {
//     //       yield* Effect.log(`increasing ${transfer.baseToken} allowance...`)

//     //       // Approve exact amount needed
//     //       const txHash = yield* increaseErc20Allowance(
//     //         transfer.baseToken,
//     //         UCS03_ADDRESS,
//     //         transfer.baseAmount
//     //       )

//     //       yield* Effect.log(`approval transaction sent: ${txHash}`)

//     //       // Wait for transaction receipt
//     //       // const receipt = yield* waitForTransactionReceipt(txHash)

//     //       // yield* Effect.log(`approval confirmed in block: ${receipt.blockNumber}`)

//     //       // Verify new allowance
//     //       const newAllowance = yield* readErc20Allowance(
//     //         transfer.baseToken,
//     //         transfer.sender,
//     //         UCS03_ADDRESS
//     //       )

//     //       yield* Effect.log(`new ${transfer.baseToken} allowance: ${newAllowance}`)
//     //     } else {
//     //       yield* Effect.log(`${transfer.baseToken} allowance is sufficient`)
//     //     }
//     //   }

//     //   yield* Effect.log("All allowances checked and increased if needed")
//     // }).pipe(Effect.withLogSpan("allowance check and increase"))

//     // yield* Effect.gen(function* () {
//     //   yield* Effect.log("creating batch")
//     //   const batch = yield* createBatch
//     //   yield* Effect.log("batch created", JSON.stringify(batch))

//     //   // Check and increase allowances before sending the batch
//     //   yield* Effect.log("checking and increasing allowances if needed")
//     //   yield* checkAndIncreaseAllowances
//     //   yield* Effect.log("allowances verified")

//     //   yield* Effect.log("sending batch")
//     //   return yield* sendInstructionEvm(batch)
//     // }).pipe(
//     //   Effect.provideService(ViemWalletClient, { client: walletClient }),
//     //   Effect.provideService(ViemWalletClient, { client: walletClient }),
//     //   Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
//     //   Effect.provideService(CosmosChannelDestination, {
//     //     ucs03address: UCS03_ADDRESS,
//     //     channelId: CHANNEL_ID
//     //   }),
//     //   Effect.provideService(EvmChannelSource, {
//     //     ucs03address: UCS03_ADDRESS,
//     //     channelId: CHANNEL_ID
//     //   }),
//     //   Effect.provideService(ViemWalletClient, {
//     //     client: walletClient,
//     //     account: account,
//     //     chain: sepolia
//     //   })
//     // )
//   })

const transferLoop = Effect.repeat(
  Effect.gen(function* (_) {
    let config = (yield* Config).config

    const transfers: Array<TransferConfig> = config.transfers ?? []
    if (transfers.length > 0) {
      yield* Effect.log("\n========== Starting transfers tasks ==========")
      for (const task of transfers) {
        if (task.enabled) {
          // yield* Effect.retry(doTransfer(task), doTransferRetrySchedule) // Retry logic for transfer
        }
      }
    } else {
      yield* Effect.log("No transfers configured. Skipping transfer step.")
    }
    yield* Effect.log("Transfers done (or skipped). Sleeping 10 minutes...")
  }),
  Schedule.spaced("3 seconds")
)

const escrowSupplyControlLoop = Effect.repeat(
  Effect.gen(function* (_) {
    yield* Effect.log("Escrow supply control loop started")
    let config = (yield* Config).config

    const tokens = yield* fetchWrappedTokens(config.hasuraEndpoint)
  
    const evmChannelBalances = new Map<
      string,           // chainId
      Map<string, bigint>  // denom → balance
    >();
    const cosmosChannelBalances = new Map<
      string,
      Map<string, bigint>
    >();

    for (const token of tokens) {
      const srcChain = token.wrapping[0]?.unwrapped_chain.universal_chain_id
      const dstChain = token.chain.universal_chain_id


      const dstChannel = token.wrapping[0]?.destination_channel_id
      if (!srcChain || !dstChain || !dstChannel) {
        yield* Effect.log("Invalid token data. Skipping...")
        continue
      }
      const sourceChannelId = yield* fetchSourceChannelId(
        config.hasuraEndpoint,
        srcChain,
        dstChain,
        dstChannel
      )
      const srcCfg = config.chainConfig[srcChain]
      const dstCfg = config.chainConfig[dstChain]

      if(!srcCfg || !dstCfg) {
        yield* Effect.log("Invalid source or destination chain configuration. Skipping...")
        continue
      }

      if(!token.wrapping || token.wrapping.length === 0 || !token.wrapping[0]?.unwrapped_denom) {
        yield* Effect.log("No wrapping information available. Skipping...")
        continue
      }

      let srcChannelBal: bigint
      const key = token.wrapping[0]!.unwrapped_denom!;  
      const path = 0n;

      if (srcCfg.chainType === "evm") {
        const client = createPublicClient({ transport: http(srcCfg.rpc) })
        srcChannelBal = yield* EthereumChannelBalance(path, key as Hex).pipe(
          Effect.provideService(ViemPublicClientDestination, { client }),
          Effect.provideService(EvmChannelDestination, {
            ucs03address: srcCfg.zkgmAddress as Hex,
            channelId: sourceChannelId!
          })
        )
        const chainMap = evmChannelBalances.get(srcChain) ?? new Map()
        const prev = chainMap.get(key) ?? 0n
        chainMap.set(key, prev + srcChannelBal)
        evmChannelBalances.set(srcChain, chainMap)


      } else {
        const client = yield* createCosmWasmClient(srcCfg.rpc)
        
        const srcChannelBalUnknown = yield* CosmosChannelBalance(path, hexToUtf8(key as Hex)).pipe(
          Effect.provideService(CosmWasmClientDestination, { client }),
          Effect.provideService(CosmosChannelDestination, {
            ucs03address: srcCfg.zkgmAddress,
            channelId: sourceChannelId!
          }),
          Effect.tapError(e =>
            Effect.logError("Error fetching channel balance:", e))
        )
        srcChannelBal = BigInt(srcChannelBalUnknown as bigint)
        
        const chainMap = cosmosChannelBalances.get(srcChain) ?? new Map()
        const prev = chainMap.get(hexToUtf8(key as Hex)) ?? 0n
        chainMap.set(hexToUtf8(key as Hex), prev + srcChannelBal)
        cosmosChannelBalances.set(srcChain, chainMap)
      
      }
      
      let totalSupply: bigint = 0n
      if (dstCfg.chainType === "evm") {
        const client = createPublicClient({ transport: http(dstCfg.rpc) })
        totalSupply = yield* readErc20TotalSupply(token.denom).pipe(
          Effect.provideService(ViemPublicClientContext, { client })
        )
      } else {
        const client = yield* createCosmWasmClient(dstCfg.rpc)
        totalSupply = BigInt(yield* readCw20TotalSupply(hexToUtf8(token.denom)).pipe(
          Effect.provideService(CosmWasmClientContext, { client })
        ))
      }

      if(srcChannelBal < totalSupply) {
        const logEffect = Effect.annotateLogs({
          issueType: "TOTAL SUPPLY IS HIGHER THAN SOURCE CHANNEL BALANCE",
          sourceChain: `${srcChain}`,
          destinationChain: `${dstChain}`,
          denom: `${token.denom}`,
          unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
          sourceChannelId: `${sourceChannelId}`,
          sourceChannelBal: `${srcChannelBal}`,
          totalSupply: `${totalSupply}`,
          destinationChannelId: `${dstChannel}`,
        })(Effect.logError(`SUPPLY ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
      } else {
        const logEffect = Effect.annotateLogs({
          sourceChain: `${srcChain}`,
          destinationChain: `${dstChain}`,
          denom: `${token.denom}`,
          unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
          sourceChannelId: `${sourceChannelId}`,
          sourceChannelBal: `${srcChannelBal}`,
          totalSupply: `${totalSupply}`,
          destinationChannelId: `${dstChannel}`,
        })(Effect.logInfo(`Channel balance is higher or equal, which is expected.`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
      }
      
    }

    yield* Effect.log("Comparing aggregated channel balances to on‑chain holdings")

    for (const [chainId, { rpc, chainType, minter }] of Object.entries(config.chainConfig)) {
      if (chainType === "evm") {
        // — EVM: use your evmChannelBalances map —
        const client = createPublicClient({
          transport: http(rpc),
        });

        for (const [tokenAddr, channelSum] of evmChannelBalances.get(chainId) ?? []) {
          const onChain = yield* readErc20Balance(tokenAddr as Hex, minter as Hex).pipe(
            Effect.provideService(ViemPublicClientContext, { client }),
            Effect.tapError(e =>
              Effect.logError("Error querying balanceOf:", e))
          );
          
          if (BigInt(onChain) < channelSum) {
            const errLog = Effect.annotateLogs({
              issueType: "AGGREGATE_GT_ONCHAIN",
              chainId,
              tokenAddr,
              minter,
              aggregated: channelSum.toString(),
              onChain: onChain.toString(),
            })(Effect.logError("AGGREGATE_MISMATCH"));
        
            Effect.runFork(errLog.pipe(Effect.provide(Logger.json)));
          } else {
            const okLog = Effect.annotateLogs({
              chainId,
              tokenAddr,
              minter,
              aggregated: channelSum.toString(),
              onChain: onChain.toString(),
            })(Effect.logInfo("AGGREGATE_OK"));
        
            Effect.runFork(okLog.pipe(Effect.provide(Logger.json)));
          }
        }
      } else {
        // — Cosmos: use your cosmosChannelBalances map —
        const cosmosClient = yield* createCosmWasmClient(rpc);

        for (const [denom, channelSum] of cosmosChannelBalances.get(chainId) ?? []) {
          const { amount } = yield* Effect.tryPromise({
            try: () => cosmosClient.getBalance(minter, denom),
            catch: e => new Error(`bank query failed: ${e}`)
          });

          if (BigInt(amount) < channelSum) {
            const errLog = Effect.annotateLogs({
              issueType: "AGGREGATE_GT_ONCHAIN",
              chainId,
              denom,
              minter,
              aggregated: channelSum.toString(),
              onChain: amount,
            })(Effect.logError("AGGREGATE_MISMATCH"));
        
            Effect.runFork(errLog.pipe(Effect.provide(Logger.json)));
          } else {
            const okLog = Effect.annotateLogs({
              chainId,
              denom,
              minter,
              aggregated: channelSum.toString(),
              onChain: amount,
            })(Effect.logInfo("AGGREGATE_OK"));
        
            Effect.runFork(okLog.pipe(Effect.provide(Logger.json)));
          }
        }
      }
    }


  }),
  Schedule.spaced("15 minutes")
)

const fundBabylonAccounts = Effect.repeat(
  Effect.gen(function* (_) {
    yield* Effect.log("Funding babylon accounts loop started")
    let config = (yield* Config).config

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(config.signer_account_mnemonic, { prefix: "bbn" })
    )
    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025bbn")
    }
    const [senderAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    const client = yield* createSigningCosmWasmClient(
      "https://rpc.bbn-1.babylon.chain.kitchen",
      wallet,
      options
    )
    let keepContinue = true;
    if (!senderAccount || !senderAccount.address) {
      yield* Effect.logError("Sender account couldnt found!")
      return
    };
    client.getBalance(senderAccount.address, "ubbn").then(balance => {
      if(parseInt(balance.amount) < 1000000) {
        const errLog = Effect.annotateLogs({
          issueType: "SPENDER_BALANCE_LOW",
          balance: balance.amount,
          chainId: "babylon.bbn-1",
          tokenAddr: "ubbn",
          account: senderAccount.address,
        })(Effect.logError("SPENDER_BALANCE_LOW"));
    
        Effect.runFork(errLog.pipe(Effect.provide(Logger.json)));
        keepContinue = false;
      }
    })
    const fee = {
      amount: coins(500, "ubbn"), // for instance 500bbn as the fee
      gas: "200000"              // fixed gas limit
    }

    if (keepContinue) {
      const accs = yield* fetchFundableAccounts(config.hasuraEndpoint)
      for(const acc of accs) {
        const receiver = acc.receiver_display
        const result = yield* Effect.tryPromise({
          try: () =>
            client.sendTokens(
              senderAccount.address,
              receiver,
              coins(10000, "ubbn"), // send 0.01 bbn
              fee
            ),
            catch: err => {
              console.error("raw sendTokens error:", err);
              throw err;
            }
        })
        const okLog = Effect.annotateLogs({
          sentAmount: "0.01",
          chainId: "babylon.bbn-1",
          tokenAddr: "ubbn",
          account: senderAccount.address,
          receiver,
          transactionHash: result.transactionHash,
        })(Effect.logInfo("SENT_OK"));
        Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))

      }
    }
  }),
  Schedule.spaced("1 minutes")
)
/**
 * fetchPacketsUntilCutoff
 *
 * This helper function pages through packets—starting from the most recent (the first query)
 * and then using the sort_order cursor (via the Next query) until it encounters a packet whose
 * packet_send_timestamp is earlier than the provided cutoff timestamp.
 *
 * @param srcChain The source chain identifier.
 * @param dstChain The destination chain identifier.
 * @param cutoffTimestamp A string ISO date (e.g. "2025-04-09T06:44:46.971Z") acting as the lower bound.
 *                        Only packets with a send timestamp >= cutoffTimestamp will be saved.
 * @returns An Effect that resolves to an array of Packet.
 */
const fetchPacketsUntilCutoff = (
  srcChain: string,
  dstChain: string,
  cutoffTimestamp: string,
  hasuraEndpoint: string
) =>
  Effect.gen(function* () {
    let allPackets: Packet[] = []
    let cursor: string | undefined
    let continueFetching = true

    while (continueFetching) {
      let response: any

      if (cursor) {
        // Next query: use the last sort_order as a cursor.
        const queryNext = gql`
          query Next($sortOrder: String!, $srcChain: String!, $dstChain: String!) {
            v2_packets(args: {
              p_source_universal_chain_id: $srcChain,
              p_destination_universal_chain_id: $dstChain,
              p_sort_order: $sortOrder
            }) {
              packet_send_timestamp
              packet_recv_timestamp
              write_ack_timestamp
              packet_ack_timestamp
              packet_send_transaction_hash
              packet_recv_transaction_hash
              write_ack_transaction_hash
              packet_ack_transaction_hash
              sort_order
              packet_send_block_hash
              packet_hash
              timeout_timestamp
            }
          }
        `
        response = yield* Effect.tryPromise({
          try: () => request(hasuraEndpoint, queryNext, { sortOrder: cursor, srcChain, dstChain }),
          catch: error => {
            console.error("Error in second query:", error)
            throw error
          }
        })
      } else {
        // First query: no cursor (assumes API returns the most recent packets).
        const queryFirst = gql`
          query First($srcChain: String!, $dstChain: String!) {
            v2_packets(args: {
              p_source_universal_chain_id: $srcChain,
              p_destination_universal_chain_id: $dstChain
            }) {
              packet_send_timestamp
              packet_recv_timestamp
              write_ack_timestamp
              packet_ack_timestamp
              packet_send_transaction_hash
              packet_recv_transaction_hash
              write_ack_transaction_hash
              packet_ack_transaction_hash
              sort_order
              packet_send_block_hash
              packet_hash
              timeout_timestamp
            }
          }
        `
        response = yield* Effect.tryPromise({
          try: () => request(hasuraEndpoint, queryFirst, { srcChain, dstChain }),
          catch: error => {
            console.info("Error in first query:", error)
            throw error
          }
        })
      }

      const currentPage: Packet[] = response?.v2_packets || []
      if (currentPage.length === 0) break

      for (const packet of currentPage) {
        // If the packet's send timestamp is missing, include it (or decide otherwise).
        if (packet.packet_send_timestamp) {
          const packetTime = new Date(packet.packet_send_timestamp).getTime()
          const cutoffTime = new Date(cutoffTimestamp).getTime()
          // Stop paging once we encounter a packet older than the cutoff.
          if (packetTime < cutoffTime) {
            continueFetching = false
            break
          }
        }
        allPackets.push(packet)
      }

      // Set cursor for the next page based on the last packet.
      if (continueFetching) {
        cursor = currentPage[currentPage.length - 1]!.sort_order
      }
    }

    return allPackets
  })

/**
 * checkPackets
 *
 * This effectful function fetches IBC packet data from Hasura and then verifies that:
 *
 *  - Packets older than the provided timeframe have a valid reception, write_ack, and ack.
 *  - If a packet’s timestamp differences exceed the provided SLA timeframe, it logs an error.
 *  - It avoids duplicate logging by tracking already reported packet send transaction hashes.
 *
 * @param sourceChain - The source chain identifier
 * @param destinationChain - The destination chain identifier
 * @param timeframeMs - The maximum allowed timeframe (in milliseconds) for the packet to be confirmed
 */
export const checkPackets = (
  sourceChain: string,
  destinationChain: string,
  timeframeMs: number,
  hasuraEndpoint: string
) =>
  Effect.gen(function* () {
    const now = Date.now()
    const searchRangeMs = timeframeMs * 5
    const sinceDate = new Date(now - searchRangeMs).toISOString()

    yield* Effect.log(
      `Querying Hasura for packets >= ${sinceDate}, chain-pair: ${sourceChain} <-> ${destinationChain}`
    )

    const now_as_date = new Date(now).toISOString()
    yield* Effect.log(`now: ${now_as_date}`)

    const packets: Packet[] = yield* fetchPacketsUntilCutoff(
      sourceChain,
      destinationChain,
      sinceDate,
      hasuraEndpoint
    )
    // const second_packets: Packet[] = yield* fetchPacketsUntilCutoff(
    //   destinationChain,
    //   sourceChain,
    //   sinceDate,
    //   hasuraEndpoint
    // )
    // const packets = [...first_packets, ...second_packets]
    yield* Effect.log(
      `Fetched ${packets.length} packets from Hasura from ${sourceChain} to ${destinationChain}`
    )
    // Process each packet.
    for (const p of packets) {
      if (!p.packet_send_timestamp) continue
      const sendTimeMs = new Date(p.packet_send_timestamp).getTime()
      // Only process packets that are older than the allowed timeframe.
      if (now - sendTimeMs < timeframeMs) continue

      if (now * 1000000 > BigInt(p.timeout_timestamp)) {
        continue
      }
      const sendTxHash = p.packet_send_transaction_hash ?? "?"
      const sort_order_tx = p.sort_order.split("-")[1]

      // 1) RECV check.
      if (p.packet_recv_timestamp) {
        const recvTimeMs = new Date(p.packet_recv_timestamp).getTime()
        if (recvTimeMs - sendTimeMs > timeframeMs) {
          // yield* Effect.log(
          //   `[RECV TOO LATE] >${timeframeMs}ms. send_time=${p.packet_send_timestamp}, recv_time=${p.packet_recv_timestamp}, sendTxHash=${sendTxHash}`
          // )
          // reportedSendTxHashes.add(sendTxHash)
        }
      } else {
        const logEffect = Effect.annotateLogs({
          issueType: "RECV_MISSING",
          sendTxHash,
          sourceChain: `${sourceChain}`,
          destinationChain: `${destinationChain}`,
          explorerUrl: `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
          minutesPassed: `${timeframeMs / 60 / 1000}`,
          packetSendBlockHash: p.packet_send_block_hash,
          packetHash: p.packet_hash,
          timeoutTimestamp: p.timeout_timestamp
        })(Effect.logError(`TRANSFER_ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

        continue
      }

      // No need to check write_ack & ack for now. Uncomment them later.

      // 2) WRITE_ACK check.
      if (p.write_ack_timestamp) {
        // const writeAckTimeMs = new Date(p.write_ack_timestamp).getTime()
        // if (writeAckTimeMs - sendTimeMs > timeframeMs) {
        //   yield* Effect.log(
        //     `[TRANSFER_ERROR: WRITE_ACK TOO LATE] >${timeframeMs}ms. send_time=${p.packet_send_timestamp}, write_ack_time=${p.write_ack_timestamp}, sendTxHash=${sendTxHash}`
        //   )
        //   reportedSendTxHashes.add(sendTxHash)
        // }
      } else {
        const logEffect = Effect.annotateLogs({
          issueType: "WRITE_ACK MISSING",
          sendTxHash,
          sourceChain: `${sourceChain}`,
          destinationChain: `${destinationChain}`,
          explorerUrl: `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
          minutesPassed: `${timeframeMs / 60 / 1000}`,
          packetSendBlockHash: p.packet_send_block_hash,
          packetHash: p.packet_hash,
          timeoutTimestamp: p.timeout_timestamp
        })(Effect.logError(`TRANSFER_ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

        continue
      }

      // 3) ACK check.
      if (p.packet_ack_timestamp) {
        // const ackTimeMs = new Date(p.packet_ack_timestamp).getTime()
        // if (ackTimeMs - sendTimeMs > timeframeMs) {
        //   yield* Effect.log(
        //     `[TRANSFER_ERROR: ACK TOO LATE] >${timeframeMs}ms. send_time=${p.packet_send_timestamp}, ack_time=${p.packet_ack_timestamp}, sendTxHash=${sendTxHash}`
        //   )
        //   reportedSendTxHashes.add(sendTxHash)
        // }
      } else {
        const logEffect = Effect.annotateLogs({
          issueType: "ACK MISSING",
          sendTxHash,
          sourceChain: `${sourceChain}`,
          destinationChain: `${destinationChain}`,
          explorerUrl: `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
          minutesPassed: `${timeframeMs / 60 / 1000}`,
          packetSendBlockHash: p.packet_send_block_hash,
          packetHash: p.packet_hash,
          timeoutTimestamp: p.timeout_timestamp
        })(Effect.logError(`TRANSFER_ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

        continue
      }
    }
  }).pipe(Effect.withLogSpan("checkPackets"))

  const runIbcChecksForever = Effect.gen(function* (_) {
    const { config } = yield* Config
  
    const schedule = Schedule.spaced(`${config.cycleIntervalMs / 1000 / 60} minutes`)
  
    const effectToRepeat = Effect.gen(function* (_) {
      const chainPairs: Array<ChainPair> = config.interactions
  
      yield* Effect.log("\n========== Starting IBC cross-chain checks ==========")
      for (const pair of chainPairs) {
        if (!pair.enabled) {
          yield* Effect.log("Checking task is disabled. Skipping.")
          continue
        }
        yield* Effect.log(
          `Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`
        )
  
        yield* checkPackets(
          pair.sourceChain,
          pair.destinationChain,
          pair.timeframeMs,
          config.hasuraEndpoint
        )
      }
  
      yield* Effect.log(
        `IBC Checks done (or skipped). Sleeping ${config.cycleIntervalMs / 1000 / 60} minutes...`
      )
      
    })
  
    return yield* Effect.repeat(effectToRepeat, schedule)
  })



const mainEffect = Effect.gen(function* (_) {
  const argv = yield* Effect.sync(() =>
    yargs(hideBin(process.argv))
      .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file"
      })
      .help()
      .alias("help", "h")
      .parseSync()
  )

  const config = yield* loadConfig(argv.config)

  yield* Effect.log("hasuraEndpoint: ", config.hasuraEndpoint)

  yield* Effect.all([/*transferLoop, */ runIbcChecksForever, escrowSupplyControlLoop, fundBabylonAccounts], {
    concurrency: "unbounded"
  }).pipe(Effect.provideService(Config, { config }))
})

Effect.runPromise(mainEffect).catch(err => Effect.logError("Error in mainEffect", err))
