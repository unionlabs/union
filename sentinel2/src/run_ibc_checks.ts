import { Effect, Logger, Schedule } from "effect"

import { gql, request } from "graphql-request"

import type { Packet } from "./sentinel2.js"
import { hasErrorOpen, markTransferError, getOpenErrors, clearTransferError} from "./db_queries.js"
import { Config, triggerIncident, resolveIncident } from "./helpers.js"
import { db } from "./sentinel2.js"

process.on("uncaughtException", err => {
  console.error("❌ Uncaught Exception:", err.stack || err)
})
process.on("unhandledRejection", (reason, promise) => {
  console.error("❌ Unhandled Rejection at:", promise, "reason:", reason)
})
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


// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
export const runIbcChecksForever = Effect.gen(function*(_) {
  const { config } = yield* Config

  const schedule = Schedule.spaced(`${config.cycleIntervalMs / 1000 / 60} minutes`)

  const effectToRepeat = Effect.gen(function*(_) {
    yield* Effect.log("\n========== Starting IBC cross-chain checks ==========")

    yield* checkPackets(
      config.hasuraEndpoint,
      config.betterstack_api_key,
      config.trigger_betterstack,
      config.isLocal,
    )
  })

  return yield* Effect.repeat(effectToRepeat, schedule)
})


export const checkPackets = (
  hasuraEndpoint: string,
  betterstack_api_key: string,
  trigger_betterstack: boolean,
  isLocal: boolean,
) =>
  Effect.gen(function*() {
    for (const sla of ["mainnet", "testnet"] as const) {
      if (sla == "testnet") {
        yield* Effect.log("Skipping testnet")
        continue
      }
      const transfer_error = sla === "mainnet" ? "MAINNET_TRANSFER_ERROR" : "TESTNET_TRANSFER_ERROR"
      const missingPacketsMainnet = yield* fetchMissingPackets(hasuraEndpoint, sla)
      if (!missingPacketsMainnet || missingPacketsMainnet.length === 0) {
        yield* Effect.log(`No missing packets found for ${sla}`)
        continue
      }
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
            trigger_betterstack,
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
          const didResolve = yield* resolveIncident(
            incident_id,
            betterstack_api_key,
            trigger_betterstack,
            isLocal,
            "Sentinel-Automatically resolved.",
          )
          if (didResolve) {
            clearTransferError(db, sla, packet_hash)
          }
        }
      }
    }
  }).pipe(Effect.withLogSpan("checkPackets"))

