import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { coins } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { FetchHttpClient } from "@effect/platform"
import {
  channelBalanceAtBlock as EthereumChannelBalanceAtBlock,
  EvmChannelDestination,
  readErc20BalanceAtBlock,
  readErc20TotalSupplyAtBlock,
  ViemPublicClient as ViemPublicClientContext,
  ViemPublicClientDestination,
} from "@unionlabs/sdk/evm"
import { Context, Data, Effect, Logger, Schedule } from "effect"
import { pipe } from "effect"
import * as Cause from "effect/Cause"
import tls from "node:tls"
import { createPublicClient, http } from "viem"

import {
  channelBalanceAtHeight,
  channelBalanceAtHeight as CosmosChannelBalanceAtHeight,
  CosmosChannelDestination,
  CosmWasmClientContext,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  getBalanceAtHeight,
  getChainHeight,
  readCw20BalanceAtHeight,
  readCw20TokenInfo,
  readCw20TotalSupplyAtHeight,
} from "@unionlabs/sdk/cosmos"

import Database from "better-sqlite3"
import type { Database as BetterSqlite3Database } from "better-sqlite3"
import { gql, request } from "graphql-request"
import fetch from "node-fetch"
import fs from "node:fs"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import { runIbcChecksForever } from "./run_ibc_checks.js"
import type { WrappedToken } from "./sentinel2.js"
import { clearPendingSupply, clearSupplyIncident, getPendingSupply, markPendingSupply, getAggregateIncident, markAggregateIncident, getSupplyIncident, markSupplyIncident, clearAggregateIncident } from "./db_queries.js"
import { hexToUtf8, Config, triggerIncident, resolveIncident } from "./helpers.js"
import type { Hex } from "./helpers.js"
import { db } from "./sentinel2.js"


interface ChannelInfo {
    source_channel_id: number
  }

/**
 * Checks whether a denom is a native token or CW20.
 * @param denom The denom address to check.
 * @returns An Effect that resolves to true if native, false if CW20.
 */
export const isDenomNative = (denom: string) =>
Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client

    return yield* readCw20TokenInfo(denom).pipe(
    Effect.provideService(CosmWasmClientContext, { client }),
    Effect.map(() => false), // If query succeeds => CW20 => false
    Effect.catchAllCause(() => Effect.succeed(true)), // If fails => native => true
    )
})

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



export const escrowSupplyControlLoop = Effect.repeat(
    Effect.gen(function*(_) {
      yield* Effect.log("Escrow supply control loop started")
      let config = (yield* Config).config
  
      const tokens = yield* fetchWrappedTokens(config.hasuraEndpoint)
  
      const evmChannelBalances = new Map<
        string, // chainId
        Map<string, bigint> // denom → balance
      >()
      const blockNumbers = new Map<string, bigint>()
      const cosmosChannelBalances = new Map<string, Map<string, bigint>>()
      if (tokens) {
        for (const { rpc, chainType } of Object.values(config.chainConfig)) {
          if (chainType === "evm") {
            const latest = yield* Effect.tryPromise({
              try: () => {
                const client = createPublicClient({ transport: http(rpc) })
                return client.getBlockNumber()
              },
              catch: e => new Error(`Failed to fetch blockNumber for ${rpc}: ${String(e)}`),
            })
            blockNumbers.set(rpc, BigInt(latest))
          } else {
            const client = yield* createCosmWasmClient(rpc).pipe(
              Effect.catchAllCause((cause) => {
                console.error(`Error fetching channel balance: ${Cause.pretty(cause)}`)
                return Effect.succeed(null)
              }),
            )
            if (!client) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No client found for rpc:",
                rpc,
              )
              continue
            }
            const latest = yield* getChainHeight(client)
            blockNumbers.set(rpc, BigInt(latest))
          }
        }
        yield* Effect.log("Fetched wrapped tokens length:", tokens.length)
        for (const token of tokens) {
          const srcChain = token.wrapping[0]?.unwrapped_chain.universal_chain_id
          const dstChain = token.chain.universal_chain_id
  
          const dstChannel = token.wrapping[0]?.destination_channel_id
          // biome-ignore lint/complexity/useSimplifiedLogicExpression: <explanation>
          if (!srcChain || !dstChain || !dstChannel) {
            yield* Effect.log(
              "[escrowSupplyControlLoop continue loop] Invalid token data. Skipping...",
            )
            continue
          }
          const sourceChannelId = yield* fetchSourceChannelId(
            config.hasuraEndpoint,
            srcChain,
            dstChain,
            dstChannel,
          )
          if (!sourceChannelId) {
            yield* Effect.log(
              "[escrowSupplyControlLoop continue loop] No source channel ID found. Skipping...",
            )
            continue
          }
  
          const srcCfg = config.chainConfig[srcChain]
          const dstCfg = config.chainConfig[dstChain]
  
          if (!(srcCfg && dstCfg)) {
            yield* Effect.log(
              "[escrowSupplyControlLoop continue loop] Invalid source or destination chain configuration. Skipping... srcChain:",
              srcChain,
              "dstChain:",
              dstChain,
            )
            continue
          }
  
          if (!token.wrapping || token.wrapping.length === 0 || !token.wrapping[0]?.unwrapped_denom) {
            yield* Effect.log(
              "[escrowSupplyControlLoop continue loop] No wrapping information available. Skipping...",
            )
            continue
          }
  
          let srcChannelBal: bigint
          // biome-ignore lint/style/noNonNullAssertion: <explanation>
          const key = token.wrapping[0]!.unwrapped_denom!
          const path = 0n
  
          if (srcCfg.chainType === "evm") {
            const client = createPublicClient({ transport: http(srcCfg.rpc) })
            const evmHeight = blockNumbers.get(srcCfg.rpc)!
            if (!evmHeight) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No block number found for source chain:",
                srcChain,
              )
              continue
            }
            const srcChannelBalHere = yield* EthereumChannelBalanceAtBlock(
              path,
              key as Hex,
              evmHeight,
            ).pipe(
              Effect.provideService(ViemPublicClientDestination, { client }),
              Effect.provideService(EvmChannelDestination, {
                ucs03address: srcCfg.zkgmAddress as Hex,
                // biome-ignore lint/style/noNonNullAssertion: <explanation>
                channelId: sourceChannelId!,
              }),
              Effect.catchAllCause((cause) => {
                console.error(`Error fetching channel balance: ${Cause.pretty(cause)}`)
                return Effect.succeed(null)
              }),
            )
            if (!srcChannelBalHere) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No srcChannelBal for token:",
                token.denom,
              )
              continue
            }
            srcChannelBal = BigInt(srcChannelBalHere as bigint)
            const chainMap = evmChannelBalances.get(srcChain) ?? new Map()
            const prev = chainMap.get(key) ?? 0n
            chainMap.set(key, prev + srcChannelBal)
            evmChannelBalances.set(srcChain, chainMap)
          } else {
            const cosmosHeight = blockNumbers.get(srcCfg.rpc)!
            if (!cosmosHeight) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No block number found for cosmos - source chain:",
                srcChain,
              )
              continue
            }
  
            const srcChannelBalUnknown = yield* channelBalanceAtHeight(
              srcCfg.restUrl,
              path,
              hexToUtf8(key as Hex),
              Number(cosmosHeight),
            ).pipe(
              Effect.provideService(CosmosChannelDestination, {
                ucs03address: srcCfg.zkgmAddress,
                // biome-ignore lint/style/noNonNullAssertion: <explanation>
                channelId: sourceChannelId!,
              }),
              Effect.catchAllCause((cause) => {
                console.error(`Error fetching channel balance: ${Cause.pretty(cause)}`)
                return Effect.succeed(null)
              }),
            )
  
            if (!srcChannelBalUnknown) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No srcChannelBalUnknown for token:",
                token.denom,
                "rest:",
                srcCfg.restUrl,
                "path:",
                path,
                "key:",
                key,
                "cosmosHeight:",
                cosmosHeight,
                "sourceChannelId:",
                sourceChannelId,
                "zkgmAddress:",
                srcCfg.zkgmAddress,
              )
              continue
            }
            srcChannelBal = BigInt(srcChannelBalUnknown)
  
            const chainMap = cosmosChannelBalances.get(srcChain) ?? new Map()
            const prev = chainMap.get(hexToUtf8(key as Hex)) ?? 0n
            chainMap.set(hexToUtf8(key as Hex), prev + srcChannelBal)
            cosmosChannelBalances.set(srcChain, chainMap)
          }
  
          let totalSupply = 0n
          if (dstCfg.chainType === "evm") {
            const client = createPublicClient({ transport: http(dstCfg.rpc) })
            const evmHeight = blockNumbers.get(dstCfg.rpc)!
            if (!evmHeight) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No block number found for destination chain:",
                dstChain,
              )
              continue
            }
            const totalSupplyHere = yield* readErc20TotalSupplyAtBlock(token.denom, evmHeight).pipe(
              Effect.provideService(ViemPublicClientContext, { client }),
              Effect.catchAllCause((cause) => {
                console.error(
                  `Failed to fetch total supply for token ${token.denom}: ${Cause.pretty(cause)}`,
                )
                return Effect.succeed(null)
              }),
            )
  
            if (!totalSupplyHere) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No total supply found for token:",
                token.denom,
              )
              continue
            }
            totalSupply = BigInt(totalSupplyHere as bigint)
          } else {
            const cosmosHeight = blockNumbers.get(dstCfg.rpc)!
            if (!cosmosHeight) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No block number found for cosmos - destination chain:",
                dstChain,
              )
              continue
            }
  
            const totalSupplyHere = yield* readCw20TotalSupplyAtHeight(
              dstCfg.restUrl,
              hexToUtf8(token.denom),
              Number(cosmosHeight),
            ).pipe(
              Effect.catchAllCause((cause) => {
                console.error(`Error fetching total supply: ${Cause.pretty(cause)}`)
                return Effect.succeed(null)
              }),
            )
            if (!totalSupplyHere) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No total supply found for token:",
                token.denom,
              )
              continue
            }
            totalSupply = BigInt(totalSupplyHere)
          }
  
          const supplyKey = `${srcChain}:${dstChain}:${token.denom}`
          const existingSupplyIncident = getSupplyIncident(db, supplyKey)
  
          const wasPending = getPendingSupply(db, supplyKey)
  
          if (srcChannelBal < totalSupply) {
            if (!wasPending) {
              markPendingSupply(db, supplyKey)
  
              const logEffect = Effect.annotateLogs({
                sourceChain: srcChain,
                destinationChain: dstChain,
                denom: token.denom,
                sourceChannelBal: srcChannelBal.toString(),
                totalSupply: totalSupply.toString(),
              })(Effect.logInfo(`SUPPLY_FIRST_FAILURE_PENDING @ ${supplyKey}`))
              Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
            } else {
              if (!existingSupplyIncident) {
                const inc = yield* triggerIncident(
                  `SUPPLY_ERROR @ ${supplyKey}`,
                  JSON.stringify({
                    issueType: "TOTAL_SUPPLY_GT_CHANNEL_BALANCE",
                    sourceChain: srcChain,
                    destinationChain: dstChain,
                    denom: token.denom,
                    unwrappedDenom: token.wrapping[0]?.unwrapped_denom,
                    sourceChannelId,
                    sourceChannelBal: srcChannelBal.toString(),
                    totalSupply: totalSupply.toString(),
                  }),
                  config.betterstack_api_key,
                  config.trigger_betterstack,
                  "SENTINEL@union.build",
                  "TOTAL_SUPPLY_GT_CHANNEL_BALANCE",
                  "Union",
                  config.isLocal,
                )
                if (inc.data.id) {
                  markSupplyIncident(db, supplyKey, inc.data.id)
                }
              }
  
              clearPendingSupply(db, supplyKey)
  
              const logEffect = Effect.annotateLogs({
                sourceChain: srcChain,
                destinationChain: dstChain,
                denom: token.denom,
                sourceChannelBal: srcChannelBal.toString(),
                totalSupply: totalSupply.toString(),
              })(Effect.logError(`SUPPLY_SECOND_FAILURE_TRIGGER @ ${supplyKey}`))
              Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
            }
          } else {
            if (wasPending) {
              clearPendingSupply(db, supplyKey)
              const logEffect = Effect.annotateLogs({
                sourceChain: srcChain,
                destinationChain: dstChain,
                denom: token.denom,
              })(Effect.logInfo(`SUPPLY_RECOVERED_CLEARED_PENDING @ ${supplyKey}`))
              Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
            }
            if (existingSupplyIncident) {
              const didResolve = yield* resolveIncident(
                existingSupplyIncident,
                config.betterstack_api_key,
                config.trigger_betterstack,
                config.isLocal,
                "Sentinel: supply back in sync",
              )
              if (didResolve) {
                clearSupplyIncident(db, supplyKey)
              }
            }
  
            const logEffect = Effect.annotateLogs({
              sourceChain: `${srcChain}`,
              destinationChain: `${dstChain}`,
              denom: `${token.denom}`,
              unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
              sourceChannelId: `${sourceChannelId}`,
              sourceChannelBal: `${srcChannelBal}`,
              totalSupply: `${totalSupply}`,
              destinationChannelId: `${dstChannel}`,
            })(Effect.logInfo(`SUPPLY_OK`))
  
            Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
          }
        }
  
        yield* Effect.log("Comparing aggregated channel balances to on‑chain holdings")
  
        for (
          const [chainId, { rpc, restUrl, chainType, minter }] of Object.entries(
            config.chainConfig,
          )
        ) {
          if (chainType === "evm") {
            const client = createPublicClient({
              transport: http(rpc),
            })
  
            const evmHeight = blockNumbers.get(rpc)!
            if (!evmHeight) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No block number found for source chain:",
                chainId,
              )
              continue
            }
  
            for (const [tokenAddr, channelSum] of evmChannelBalances.get(chainId) ?? []) {
              const onChainRaw = yield* readErc20BalanceAtBlock(
                tokenAddr as Hex,
                minter as Hex,
                evmHeight,
              ).pipe(
                Effect.provideService(ViemPublicClientContext, { client }),
                Effect.catchAllCause((cause) => {
                  console.error(`Error querying balanceOf: ${Cause.pretty(cause)}`)
                  return Effect.succeed(null)
                }),
              )
              if (!onChainRaw) {
                yield* Effect.log(
                  "[escrowSupplyControlLoop continue loop] No balance found for denom:",
                  tokenAddr,
                )
                continue
              }
              const onChain = BigInt(onChainRaw as bigint)
              const aggregateKey = `${chainId}:${tokenAddr}`
              const existingAgg = getAggregateIncident(db, aggregateKey)
  
              if (onChain < channelSum) {
                if (!existingAgg) {
                  const inc = yield* triggerIncident(
                    `AGGREGATE_MISMATCH @ ${aggregateKey}`,
                    JSON.stringify({
                      issueType: "AGGREGATE_GT_ONCHAIN",
                      chainId,
                      tokenAddr,
                      minter,
                      aggregated: channelSum.toString(),
                      onChain: onChain.toString(),
                    }),
                    config.betterstack_api_key,
                    config.trigger_betterstack,
                    "SENTINEL@union.build",
                    "AGGREGATE_GT_ONCHAIN",
                    "Union",
                    config.isLocal,
                  )
                  if (inc.data.id) {
                    markAggregateIncident(db, aggregateKey, inc.data.id)
                  }
                }
  
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
                if (existingAgg) {
                  const didResolve = yield* resolveIncident(
                    existingAgg,
                    config.betterstack_api_key,
                    config.trigger_betterstack,
                    config.isLocal,
                    "Sentinel: aggregate back in sync",
                  )
                  if (didResolve) {
                    clearAggregateIncident(db, aggregateKey)
                  }
                }
  
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
            const cosmosClient = yield* createCosmWasmClient(rpc).pipe(
              Effect.catchAllCause((cause) => {
                console.error(`Error fetching channel balance: ${Cause.pretty(cause)}`)
                return Effect.succeed(null)
              }),
            )
            if (!cosmosClient) {
              yield* Effect.log(
                "[escrowSupplyControlLoop continue loop] No client found for rpc:",
                rpc,
              )
              continue
            }
  
            for (const [denom, channelSum] of cosmosChannelBalances.get(chainId) ?? []) {
              const isDenomNativeHere = yield* isDenomNative(denom).pipe(
                Effect.provideService(CosmWasmClientContext, { client: cosmosClient }),
                Effect.catchAllCause((cause) => {
                  console.error(`Error checking denom type: ${Cause.pretty(cause)}`)
                  return Effect.succeed(null)
                }),
              )
              let amount
              const cosmosHeight = blockNumbers.get(rpc)!
              if (!cosmosHeight) {
                yield* Effect.log(
                  "[escrowSupplyControlLoop continue loop] No block number found for cosmos - chain:",
                  chainId,
                )
                continue
              }
              if (isDenomNativeHere) {
                // const balance = yield* Effect.tryPromise({
                //   try: () => cosmosClient.getBalance(minter, denom),
                //   catch: e => new Error(`bank query failed: ${e}`),
                // })
                const balance = yield* getBalanceAtHeight(
                  restUrl,
                  minter,
                  denom,
                  Number(cosmosHeight),
                ).pipe(
                  Effect.provide(FetchHttpClient.layer),
                  Effect.catchAllCause((cause) => {
                    console.error(`Error fetching channel balance at height: ${Cause.pretty(cause)}`)
                    return Effect.succeed(null)
                  }),
                )
                if (!balance) {
                  yield* Effect.log(
                    "[escrowSupplyControlLoop continue loop] No balance found for denom:",
                    denom,
                  )
                  continue
                }
                amount = BigInt(balance)
              } else {
                const balance = yield* readCw20BalanceAtHeight(
                  restUrl,
                  denom,
                  minter,
                  Number(cosmosHeight),
                ).pipe(
                  Effect.catchAllCause((cause) => {
                    console.error(`Error fetching balance: ${Cause.pretty(cause)}`)
                    return Effect.succeed(null)
                  }),
                )
                if (!balance) {
                  yield* Effect.log(
                    "[escrowSupplyControlLoop continue loop] No balance found for denom:",
                    denom,
                  )
                  continue
                }
                amount = BigInt(balance)
              }
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
      }
    }).pipe(
      Effect.catchAllCause(err =>
        Effect.sync(() => {
          console.error("⚠️ escrowSupplyControlLoop iteration failed, skipping:", err)
        })
      ),
    ),
    Schedule.spaced("1 hours"),
  )
  
  