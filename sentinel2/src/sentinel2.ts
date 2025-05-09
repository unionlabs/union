import type { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { coins } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { Context, Data, Effect, Logger, Schedule } from "effect"
import { createPublicClient, http } from "viem"

import {
  channelBalance as EthereumChannelBalance,
  EvmChannelDestination,
  readErc20Balance,
  readErc20TotalSupply,
  ViemPublicClient as ViemPublicClientContext,
  ViemPublicClientDestination,
} from "@unionlabs/sdk/evm"

import {
  channelBalance as CosmosChannelBalance,
  CosmosChannelDestination,
  CosmWasmClientContext,
  CosmWasmClientDestination,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  readCw20TotalSupply,
} from "@unionlabs/sdk/cosmos"

import Database from "better-sqlite3"
import type { Database as BetterSqlite3Database } from "better-sqlite3"
import { gql, request } from "graphql-request"
import fetch from "node-fetch"
import fs from "node:fs"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

type Hex = `0x${string}`
let db: BetterSqlite3Database

/**
 * Effect to trigger a BetterStack incident via the Uptime API
 */
export const triggerIncident = (
  summary: string,
  description: string,
  apiKey: string,
  requesterEmail: string,
  incidentName: string,
  teamName: string,
  isLocal: boolean,
) =>
  isLocal
    ? Effect.sync(() => {
      console.info("Local mode: skipping triggerIncident")
      return { data: { id: "" } }
    })
    : Effect.tryPromise({
      try: () =>
        fetch("https://uptime.betterstack.com/api/v3/incidents", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${apiKey}`,
          },
          body: JSON.stringify({
            summary,
            description,
            requester_email: requesterEmail,
            ...(teamName ? { team_name: teamName } : {}),
            call: false,
            sms: false,
            email: false,
            name: incidentName,
          }),
        }).then(async res => {
          const text = await res.text()
          if (!res.ok) {
            throw new Error(`Trigger failed: ${text}`)
          }
          return JSON.parse(text)
        }),
      catch: e => new Error(`Incident trigger error: ${e}`),
    })

/**
 * Effect to resolve an existing BetterStack incident via the Uptime API
 */
export const resolveIncident = (
  incidentId: string,
  apiKey: string,
  isLocal: boolean,
  resolvedBy = "SENTINEL@union.build",
) =>
  isLocal
    ? Effect.sync(() => {
      console.info("Local mode: skipping resolveIncident")
      return { data: { id: incidentId } }
    })
    : Effect.tryPromise({
      try: () =>
        fetch(`https://uptime.betterstack.com/api/v3/incidents/${incidentId}/resolve`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${apiKey}`,
          },
          body: JSON.stringify({ resolved_by: resolvedBy }),
        }).then(async res => {
          const text = await res.text()
          if (!res.ok) {
            throw new Error(`Resolve failed: ${text}`)
          }
          return JSON.parse(text)
        }),
      catch: e => new Error(`Incident resolve error: ${e}`),
    })

export function isFunded(db: BetterSqlite3Database, txHash: string) {
  const row = db.prepare(`SELECT 1 FROM funded_txs WHERE transaction_hash = ?`).get(txHash)
  return !!row
}

export function getSignerIncident(db: BetterSqlite3Database, key: string): string | undefined {
  const row = db.prepare(`SELECT incident_id FROM signer_incidents WHERE key = ?`).get(key) as
    | { incident_id: string }
    | undefined
  return row?.incident_id
}

export function markSignerIncident(db: BetterSqlite3Database, key: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO signer_incidents
      (key, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(key, incidentId)
}

export function clearSignerIncident(db: BetterSqlite3Database, key: string) {
  db.prepare(`DELETE FROM signer_incidents WHERE key = ?`).run(key)
}

export function addFunded(db: BetterSqlite3Database, txHash: string) {
  db.prepare(`INSERT OR IGNORE INTO funded_txs (transaction_hash) VALUES (?)`).run(txHash)
}

export function getIncidentId(db: BetterSqlite3Database, packetHash: string): string | undefined {
  const row = db
    .prepare(`SELECT incident_id FROM transfer_errors WHERE packet_hash = ?`)
    .get(packetHash) as { incident_id: string } | undefined
  return row?.incident_id
}

export function hasErrorOpen(db: BetterSqlite3Database, sla: string, packetHash: string) {
  return !!db
    .prepare(
      `SELECT 1
         FROM transfer_errors
        WHERE sla = ?
          AND packet_hash = ?`,
    )
    .get(sla, packetHash)
}

export function markTransferError(
  db: BetterSqlite3Database,
  sla: string,
  packetHash: string,
  incidentId: string,
) {
  db.prepare(`
    INSERT OR REPLACE INTO transfer_errors
      (sla, packet_hash, incident_id, inserted_at)
    VALUES (?, ?, ?, strftime('%s','now')*1000)
  `).run(sla, packetHash, incidentId)
}

export function clearTransferError(db: BetterSqlite3Database, sla: string, packetHash: string) {
  db.prepare(`
    DELETE FROM transfer_errors
     WHERE sla = ?
       AND packet_hash = ?
  `).run(sla, packetHash)
}

export function getOpenErrors(
  db: BetterSqlite3Database,
  sla: string,
): Array<{ packet_hash: string; incident_id: string }> {
  return db
    .prepare(
      `SELECT packet_hash, incident_id
         FROM transfer_errors
        WHERE sla = ?`,
    )
    .all(sla) as Array<{ packet_hash: string; incident_id: string }>
}

function hexToUtf8(hex: string): string {
  // strip optional 0x
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex
  // build a Buffer from hex, then decode as UTF‑8
  return Buffer.from(clean, "hex").toString("utf8")
}

// Chain pair configuration
interface ChainPair {
  sourceChain: string
  destinationChain: string
  timeframeMs: number
  enabled: boolean
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
  source_chain: {
    universal_chain_id: string
  }
  destination_chain: {
    universal_chain_id: string
  }
  packet_send_timestamp: string
  packet_hash: string
  status: string
  sort_order: string
}

type ChainType = "evm" | "cosmos"

interface ChainConfigEntry {
  zkgmAddress: string
  rpc: string
  chainType: ChainType
  minter: string
}

type ChainConfig = Record<string, ChainConfigEntry>

export interface SignerBalanceThresholds {
  [plugin: string]: bigint
}

export type PortSignerBalances = Record<string, SignerBalanceThresholds>

export type SignerBalancesConfig = Record<string, PortSignerBalances>

// Combined configuration shape
interface ConfigFile {
  cycleIntervalMs: number
  hasuraEndpoint: string
  signerBalances: SignerBalancesConfig
  chainConfig: ChainConfig
  signer_account_mnemonic: string
  betterstack_api_key: string
  dbPath: string
  isLocal: boolean
}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

export class Config extends Context.Tag("Config")<Config, { readonly config: ConfigFile }>() {}

const fetchWrappedTokens = (hasuraEndpoint: string) =>
  Effect.gen(function*() {
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
      },
    })

    const tokens: Array<WrappedToken> = response?.v2_tokens || []
    return tokens
  })

const fetchFundableAccounts = (hasuraEndpoint: string) =>
  Effect.gen(function*() {
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
      },
    })

    const tokens: Array<FundableAccounts> = response?.v2_transfers || []
    const filtered: Array<FundableAccounts> = tokens
      .map(({ receiver_display, traces }) => ({
        receiver_display,
        traces: traces
          .filter(
            trace =>
              trace.type === "WRITE_ACK"
              && trace.transaction_hash != null
              && !isFunded(db, trace.transaction_hash),
          )
          // biome-ignore lint/style/noNonNullAssertion: <explanation>
          .map(trace => ({ type: trace.type, transaction_hash: trace.transaction_hash! })),
      }))
      .filter(acc => acc.traces.length > 0)

    return filtered
  })

const fetchSourceChannelId = (
  hasuraEndpoint: string,
  srcChain: string,
  dstChain: string,
  dstChannelId: number,
) =>
  Effect.gen(function*() {
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
      },
    })

    const channels: Array<ChannelInfo> = response?.v2_channels || []
    return channels[0]?.source_channel_id
  })

function loadConfig(configPath: string) {
  return Effect.tryPromise({
    // biome-ignore lint/suspicious/useAwait: <explanation>
    try: async () => {
      if (!fs.existsSync(configPath)) {
        throw new Error("Config file not found. Ensure config.json exists.")
      }
      const rawData = fs.readFileSync(configPath, "utf-8")
      const config: ConfigFile = JSON.parse(rawData)

      return config
    },
    catch: error =>
      new FilesystemError({
        message: "Config file is invalid.",
        cause: error,
      }),
  })
}

const escrowSupplyControlLoop = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Escrow supply control loop started")
    let config = (yield* Config).config

    const tokens = yield* fetchWrappedTokens(config.hasuraEndpoint)

    const evmChannelBalances = new Map<
      string, // chainId
      Map<string, bigint> // denom → balance
    >()
    const cosmosChannelBalances = new Map<string, Map<string, bigint>>()

    for (const token of tokens) {
      const srcChain = token.wrapping[0]?.unwrapped_chain.universal_chain_id
      const dstChain = token.chain.universal_chain_id

      const dstChannel = token.wrapping[0]?.destination_channel_id
      // biome-ignore lint/complexity/useSimplifiedLogicExpression: <explanation>
      if (!srcChain || !dstChain || !dstChannel) {
        yield* Effect.log("Invalid token data. Skipping...")
        continue
      }
      const sourceChannelId = yield* fetchSourceChannelId(
        config.hasuraEndpoint,
        srcChain,
        dstChain,
        dstChannel,
      )
      const srcCfg = config.chainConfig[srcChain]
      const dstCfg = config.chainConfig[dstChain]

      if (!(srcCfg && dstCfg)) {
        yield* Effect.log("Invalid source or destination chain configuration. Skipping...")
        continue
      }

      if (!token.wrapping || token.wrapping.length === 0 || !token.wrapping[0]?.unwrapped_denom) {
        yield* Effect.log("No wrapping information available. Skipping...")
        continue
      }

      let srcChannelBal: bigint
      // biome-ignore lint/style/noNonNullAssertion: <explanation>
      const key = token.wrapping[0]!.unwrapped_denom!
      const path = 0n

      if (srcCfg.chainType === "evm") {
        const client = createPublicClient({ transport: http(srcCfg.rpc) })
        srcChannelBal = yield* EthereumChannelBalance(path, key as Hex).pipe(
          Effect.provideService(ViemPublicClientDestination, { client }),
          Effect.provideService(EvmChannelDestination, {
            ucs03address: srcCfg.zkgmAddress as Hex,
            // biome-ignore lint/style/noNonNullAssertion: <explanation>
            channelId: sourceChannelId!,
          }),
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
            // biome-ignore lint/style/noNonNullAssertion: <explanation>
            channelId: sourceChannelId!,
          }),
          Effect.tapError(e => Effect.logError("Error fetching channel balance:", e)),
        )
        srcChannelBal = BigInt(srcChannelBalUnknown as bigint)

        const chainMap = cosmosChannelBalances.get(srcChain) ?? new Map()
        const prev = chainMap.get(hexToUtf8(key as Hex)) ?? 0n
        chainMap.set(hexToUtf8(key as Hex), prev + srcChannelBal)
        cosmosChannelBalances.set(srcChain, chainMap)
      }

      let totalSupply = 0n
      if (dstCfg.chainType === "evm") {
        const client = createPublicClient({ transport: http(dstCfg.rpc) })
        totalSupply = yield* readErc20TotalSupply(token.denom).pipe(
          Effect.provideService(ViemPublicClientContext, { client }),
        )
      } else {
        const client = yield* createCosmWasmClient(dstCfg.rpc)
        totalSupply = BigInt(
          yield* readCw20TotalSupply(hexToUtf8(token.denom)).pipe(
            Effect.provideService(CosmWasmClientContext, { client }),
          ),
        )
      }

      if (srcChannelBal < totalSupply) {
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
        const client = createPublicClient({
          transport: http(rpc),
        })

        for (const [tokenAddr, channelSum] of evmChannelBalances.get(chainId) ?? []) {
          const onChain = yield* readErc20Balance(tokenAddr as Hex, minter as Hex).pipe(
            Effect.provideService(ViemPublicClientContext, { client }),
            Effect.tapError(e => Effect.logError("Error querying balanceOf:", e)),
          )

          if (BigInt(onChain) < channelSum) {
            const errLog = Effect.annotateLogs({
              issueType: "AGGREGATE_GT_ONCHAIN",
              chainId,
              tokenAddr,
              minter,
              aggregated: channelSum.toString(),
              onChain: onChain.toString(),
            })(Effect.logError("AGGREGATE_MISMATCH"))

            Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
          } else {
            const okLog = Effect.annotateLogs({
              chainId,
              tokenAddr,
              minter,
              aggregated: channelSum.toString(),
              onChain: onChain.toString(),
            })(Effect.logInfo("AGGREGATE_OK"))

            Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
          }
        }
      } else {
        const cosmosClient = yield* createCosmWasmClient(rpc)

        for (const [denom, channelSum] of cosmosChannelBalances.get(chainId) ?? []) {
          const { amount } = yield* Effect.tryPromise({
            try: () => cosmosClient.getBalance(minter, denom),
            catch: e => new Error(`bank query failed: ${e}`),
          })

          if (BigInt(amount) < channelSum) {
            const errLog = Effect.annotateLogs({
              issueType: "AGGREGATE_GT_ONCHAIN",
              chainId,
              denom,
              minter,
              aggregated: channelSum.toString(),
              onChain: amount,
            })(Effect.logError("AGGREGATE_MISMATCH"))

            Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
          } else {
            const okLog = Effect.annotateLogs({
              chainId,
              denom,
              minter,
              aggregated: channelSum.toString(),
              onChain: amount,
            })(Effect.logInfo("AGGREGATE_OK"))

            Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
          }
        }
      }
    }
  }),
  Schedule.spaced("15 minutes"),
)

const fundBabylonAccounts = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Funding babylon accounts loop started")
    let config = (yield* Config).config
    if (config.isLocal) {
      yield* Effect.log("Local mode: skipping funding babylon accounts")
      return
    }

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(config.signer_account_mnemonic, { prefix: "bbn" })
    )
    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025bbn"),
    }
    const [senderAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    const client = yield* createSigningCosmWasmClient(
      "https://rpc.bbn-1.babylon.chain.kitchen",
      wallet,
      options,
    )

    if (!senderAccount?.address) {
      yield* Effect.logError("Sender account couldnt found!")
      return
    }
    const balance = yield* Effect.tryPromise(() => client.getBalance(senderAccount.address, "ubbn"))

    if (Number.parseInt(balance.amount) < 1_000_000) {
      const errLog = Effect.annotateLogs({
        issueType: "SPENDER_BALANCE_LOW",
        balance: balance.amount,
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
      })(Effect.logError("SPENDER_BALANCE_LOW"))

      Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
      return
    }

    const fee = {
      amount: coins(500, "ubbn"),
      gas: "200000",
    }

    const accs = yield* fetchFundableAccounts(config.hasuraEndpoint)
    for (const acc of accs) {
      const receiver = acc.receiver_display
      const result = yield* Effect.tryPromise({
        try: () =>
          client.sendTokens(
            senderAccount.address,
            receiver,
            coins(10000, "ubbn"), // send 0.01 bbn
            fee,
          ),
        catch: err => {
          console.error("raw sendTokens error:", err)
          throw err
        },
      })

      addFunded(db, result.transactionHash)

      const okLog = Effect.annotateLogs({
        sentAmount: "0.01",
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
        receiver,
        transactionHash: result.transactionHash,
      })(Effect.logInfo("SENT_OK"))
      Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
    }
  }),
  Schedule.spaced("1 minutes"),
)

interface PostRequestInput {
  url: string
  port?: number
  headers: Record<string, string>
  payload: unknown
}

interface PostRequestError {
  readonly _tag: "PostRequestError"
  readonly message: string
  readonly status?: number
}

export const safePostRequest = ({ url, port, headers, payload }: PostRequestInput) => {
  const fullUrl = port ? `${url}:${port}` : url

  return Effect.tryPromise({
    try: () =>
      fetch(fullUrl, {
        method: "POST",
        headers,
        body: JSON.stringify(payload),
      }).then(async response => {
        if (response.status === 200) {
          return await response.json()
        }
        const text = await response.text().catch(() => "")
        // biome-ignore lint/style/useThrowOnlyError: <explanation>
        throw {
          _tag: "PostRequestError",
          message: `Non-200 status: ${response.status} body: ${text}`,
          status: response.status,
        }
      }),
    catch: error =>
      ({
        _tag: "PostRequestError",
        message: error instanceof Error
          ? error.message
          : typeof error === "object"
          ? JSON.stringify(error)
          : String(error),
        status: (error as any)?.status,
      }) satisfies PostRequestError,
  })
}

export const checkBalances = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Spawning per-plugin balance checks…")
    const { config } = yield* Config
    const sbConfig = config.signerBalances

    for (const [url, ports] of Object.entries(sbConfig)) {
      for (const [portStr, plugins] of Object.entries(ports)) {
        const port = Number(portStr)

        for (const [plugin, expectedThreshold] of Object.entries(plugins)) {
          const payload = [
            {
              jsonrpc: "2.0",
              id: 1,
              method: "voyager_pluginCustom",
              params: [plugin, "signerBalances", []] as const,
            },
          ]

          const callWithRetry = safePostRequest({
            url,
            port,
            headers: { "Content-Type": "application/json" },
            payload,
          }).pipe(
            // retry forever every 2 min on PostRequestError (or any thrown error)
            Effect.retry(Schedule.spaced("2 minutes")),
          )

          const worker = Effect.gen(function*(_) {
            const result = yield* callWithRetry

            if (!Array.isArray(result) || result.length === 0) {
              yield* Effect.logError(`Unexpected response shape for ${plugin} @ ${url}:${port}`)
              return
            }

            const rpcObj = (result[0] as any).result
            if (typeof rpcObj !== "object" || rpcObj === null) {
              yield* Effect.logError(`No 'result' object for ${plugin} @ ${url}:${port}`)
              return
            }

            for (const [wallet, balStr] of Object.entries(rpcObj)) {
              let bal = BigInt(balStr as string)

              const tags = {
                plugin,
                url,
                port: portStr,
                wallet,
                balance: bal.toString(),
                expected: expectedThreshold.toString(),
              }

              const key = `${url}:${port}:${plugin}:${wallet}`
              const existing = getSignerIncident(db, key)

              if (bal < expectedThreshold) {
                const logEffect = Effect.annotateLogs(tags)(Effect.logError("SIGNER_BALANCE_LOW"))
                Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

                if (!existing) {
                  const inc = yield* triggerIncident(
                    `SIGNER_BALANCE_LOW @ ${key}`,
                    JSON.stringify({ plugin, url, port: portStr, wallet, balance: bal.toString() }),
                    config.betterstack_api_key,
                    "SENTINEL@union.build",
                    "SIGNER_BALANCE_LOW",
                    "Union",
                    config.isLocal,
                  )
                  markSignerIncident(db, key, inc.data.id)
                }
              } else {
                const logEffect = Effect.annotateLogs(tags)(Effect.logInfo("SIGNER_BALANCE_OK"))
                Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

                if (existing) {
                  yield* resolveIncident(
                    existing,
                    config.betterstack_api_key,
                    config.isLocal,
                    "Sentinel-Automatically resolved.",
                  )
                  clearSignerIncident(db, key)
                }
              }
            }
          })
          Effect.runFork(worker)
        }
      }
    }
  }),
  Schedule.spaced("30 minutes"),
)

const fetchMissingPackets = (hasuraEndpoint: string, exceedingSla: string) =>
  Effect.gen(function*() {
    let allPackets: Array<Packet> = []
    let cursor: string | undefined
    let continueFetching = true

    while (continueFetching) {
      let response: any

      if (cursor) {
        const queryNext = gql`
            query MissingPacketsNext($sla: String!, $cursor: String!) {
              v2_packets(args: {
                p_exceeding_sla: $sla,
                p_sort_order: $cursor
              }) {
                source_chain { universal_chain_id }
                destination_chain { universal_chain_id }
                packet_send_timestamp
                packet_hash
                status
                sort_order
              }
            }
          `
        response = yield* Effect.tryPromise({
          try: () =>
            request(hasuraEndpoint, queryNext, {
              sla: exceedingSla,
              cursor,
            }),
          catch: err => {
            console.error("fetchMissingPackets (next) failed:", err)
            return []
          },
        })
      } else {
        const queryFirst = gql`
            query MissingPackets($sla: String!) {
              v2_packets(args: { p_exceeding_sla: $sla }) {
                source_chain { universal_chain_id }
                destination_chain { universal_chain_id }
                packet_send_timestamp
                packet_hash
                status
                sort_order
              }
            }
          `
        response = yield* Effect.tryPromise({
          try: () =>
            request(hasuraEndpoint, queryFirst, {
              sla: exceedingSla,
            }),
          catch: err => {
            console.error("fetchMissingPackets (first) failed:", err)
            return []
          },
        })
      }

      const page: Array<Packet> = response.v2_packets || []
      if (page.length === 0) {
        break
      }

      allPackets.push(...page)
      // biome-ignore lint/style/noNonNullAssertion: <explanation>
      const last = page.at(-1)!

      cursor = last.sort_order
    }

    return allPackets
  })

export const checkPackets = (
  hasuraEndpoint: string,
  betterstack_api_key: string,
  isLocal: boolean,
) =>
  Effect.gen(function*() {
    for (const sla of ["mainnet", "testnet"] as const) {
      const transfer_error = sla === "mainnet" ? "MAINNET_TRANSFER_ERROR" : "TESTNET_TRANSFER_ERROR"
      const missingPacketsMainnet = yield* fetchMissingPackets(hasuraEndpoint, sla)
      yield* Effect.log(`Fetched ${missingPacketsMainnet.length} missingPackets from Hasura`)

      for (const missingPacket of missingPacketsMainnet) {
        const whole_description = {
          issueType: "TRANSFER_FAILED",
          currentStatus: missingPacket.status,
          sourceChain: missingPacket.source_chain.universal_chain_id,
          destinationChain: missingPacket.destination_chain.universal_chain_id,
          packetSendTimestamp: missingPacket.packet_send_timestamp,
          packetHash: missingPacket.packet_hash,
          explorerUrl: `https://btc.union.build/explorer/transfers/${missingPacket.packet_hash}`,
        }
        const logEffect = Effect.annotateLogs(whole_description)(Effect.logError(transfer_error))

        if (!hasErrorOpen(db, sla, missingPacket.packet_hash)) {
          const val = yield* triggerIncident(
            `${transfer_error}: https://btc.union.build/explorer/transfers/${missingPacket.packet_hash}`,
            JSON.stringify(whole_description),
            betterstack_api_key,
            "SENTINEL@union.build",
            transfer_error,
            "Union",
            isLocal,
          )
          markTransferError(db, sla, missingPacket.packet_hash, val.data.id)
        }
        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
      }
      const openErrors = getOpenErrors(db, sla)

      const missingSet = new Set(missingPacketsMainnet.map(p => p.packet_hash))

      for (const { packet_hash, incident_id } of openErrors) {
        if (!missingSet.has(packet_hash)) {
          yield* Effect.log(`Auto-resolving incident for packet ${packet_hash}`)
          yield* resolveIncident(
            incident_id,
            betterstack_api_key,
            isLocal,
            "Sentinel-Automatically resolved.",
          )
          clearTransferError(db, sla, packet_hash)
        }
      }
    }
  }).pipe(Effect.withLogSpan("checkPackets"))

const runIbcChecksForever = Effect.gen(function*(_) {
  const { config } = yield* Config

  const schedule = Schedule.spaced(`${config.cycleIntervalMs / 1000 / 60} minutes`)

  const effectToRepeat = Effect.gen(function*(_) {
    yield* Effect.log("\n========== Starting IBC cross-chain checks ==========")

    yield* checkPackets(config.hasuraEndpoint, config.betterstack_api_key, config.isLocal)
  })

  return yield* Effect.repeat(effectToRepeat, schedule)
})

const mainEffect = Effect.gen(function*(_) {
  const argv = yield* Effect.sync(() =>
    yargs(hideBin(process.argv))
      .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file",
      })
      .help()
      .alias("help", "h")
      .parseSync()
  )

  console.info("Loading config from", argv.config)
  const config = yield* loadConfig(argv.config)

  try {
    db = new Database(config.dbPath)
  } catch (err) {
    console.error("Error opening database:", err)
    throw err
  }

  db.prepare(`
    CREATE TABLE IF NOT EXISTS funded_txs (
      transaction_hash TEXT PRIMARY KEY
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS transfer_errors (
      sla           TEXT    NOT NULL,
      packet_hash   TEXT PRIMARY KEY,
      incident_id   TEXT NOT NULL,
      inserted_at   INTEGER
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS signer_incidents (
      key          TEXT PRIMARY KEY,    -- "url:port:plugin:wallet"
      incident_id  TEXT NOT NULL,
      inserted_at  INTEGER
    )
  `).run()

  yield* Effect.log("Database opened at", config.dbPath, "hasuraEndpoint:", config.hasuraEndpoint)

  yield* Effect.all(
    [runIbcChecksForever, escrowSupplyControlLoop, fundBabylonAccounts, checkBalances],
    {
      concurrency: "unbounded",
    },
  ).pipe(Effect.provideService(Config, { config }))
})

Effect.runPromise(mainEffect).catch(err => Effect.logError("Error in mainEffect", err))
