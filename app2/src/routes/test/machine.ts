import { Effect, Schedule, Duration, Fiber, Schema } from 'effect'
import { fetchDecodeGraphql } from "$lib/utils/queries"
import { TransferList } from "@unionlabs/sdk/schema"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { runFork } from '$lib/runtime'
import type { TransferListItem } from '@unionlabs/sdk/schema'
import { graphql } from "gql.tada"

const MAINNET_ONLY = false // Set to true to only show mainnet transfers

interface TransferPollingMachine {
  onNewTransfers: (callback: (newTxs: Array<TransferListItem>) => void) => void
  destroy: () => void
}

// Initial fetch for latest transfers to establish baseline
const fetchLatestTransfers = (limit = 50, mainnetOnly = MAINNET_ONLY) =>
  fetchDecodeGraphql(
    Schema.Struct({ v2_transfers: TransferList }),
    graphql(
      `
    query TransferListLatest($limit: Int!, $network: String) @cached(ttl: 1) {
      v2_transfers(args: {
        p_limit: $limit,
        p_network: $network
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment],
    ),
    {
      limit,
      network: mainnetOnly ? "mainnet" : null,
    }
  )

// Fetch transfers newer than a given sort_order using pagination
const fetchNewTransfers = (lastSortOrder: string, limit = 50, mainnetOnly = MAINNET_ONLY) =>
  fetchDecodeGraphql(
    Schema.Struct({ v2_transfers: TransferList }),
    graphql(
      `
    query TransferListPage($page: String!, $limit: Int!, $network: String) {
      v2_transfers(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_comparison: "gt",
        p_network: $network
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment],
    ),
    {
      page: lastSortOrder,
      limit,
      network: mainnetOnly ? "mainnet" : null,
    }
  )

export function createTransferPollingMachine(limit = 50): TransferPollingMachine {
  let lastSortOrder: string | null = null
  let pollingFiber: Fiber.RuntimeFiber<number, unknown> | null = null
  let callback: ((newTxs: Array<TransferListItem>) => void) | null = null
  let isInitialFetch = true
  
  // Transfer scheduling for smooth streaming
  let scheduledTransfers: Array<{
    transfer: TransferListItem
    scheduledTime: number
  }> = []
  const FUTURE_BUFFER_SECONDS = 1
  const SPREAD_TIME_MS = 3000
  const POLL_INTERVAL_MS = 1000

  const checkForNewTransfers = Effect.gen(function* () {
          if (isInitialFetch || !lastSortOrder) {
        // STEP 1: Get latest single transfer to establish baseline (don't display it)
        const result = yield* fetchLatestTransfers(1)
        const transfers = [...result.v2_transfers]
        isInitialFetch = false
      
        if (transfers.length > 0) {
          // Set baseline to this latest transfer, so gt queries get everything newer
          lastSortOrder = transfers[0].sort_order
        }
        // Don't schedule any transfers - let polling handle all display
      } else {
        // STEP 2: Poll for transfers newer than our baseline using pagination
        try {
          const result = yield* fetchNewTransfers(lastSortOrder!, limit)
          const transfers = [...result.v2_transfers]
        
        if (transfers.length > 0) {
          // Update baseline to newest transfer found
          lastSortOrder = transfers.map((t: TransferListItem) => t.sort_order).sort().pop() ?? lastSortOrder
          scheduleTransfers(transfers)
        }
      } catch (error) {
        console.error("[DEBUG] fetchNewTransfers failed:", error)
      }
    }
  })

  const scheduleTransfers = (newTransfers: Array<TransferListItem>) => {
    const now = Date.now()
    const futureBaseTime = now + FUTURE_BUFFER_SECONDS * 1000

    newTransfers.forEach((transfer: TransferListItem, index: number) => {
      const spread = (index / Math.max(newTransfers.length - 1, 1)) * SPREAD_TIME_MS
      const jitter = (Math.random() - 0.5) * 500
      const scheduledTime = futureBaseTime + spread + jitter

      scheduledTransfers.push({ transfer, scheduledTime })
    })

    scheduledTransfers.sort((a, b) => a.scheduledTime - b.scheduledTime)
  }

  // Process scheduled transfers for smooth streaming
  const processScheduledTransfers = () => {
    const checkScheduled = () => {
      const now = Date.now()
      const ready = scheduledTransfers.filter(t => t.scheduledTime <= now)
      
      if (ready.length > 0 && callback) {
        ready.forEach(t => callback!([t.transfer]))
        scheduledTransfers = scheduledTransfers.filter(t => t.scheduledTime > now)
      }

      setTimeout(checkScheduled, 50)
    }
    checkScheduled()
  }

  const startMachine = () => {
    const pollingEffect = checkForNewTransfers.pipe(
      Effect.repeat(Schedule.spaced(Duration.millis(POLL_INTERVAL_MS)))
    )
    pollingFiber = runFork(pollingEffect)
    processScheduledTransfers()
  }

  startMachine()

  function destroy() {
    if (pollingFiber) {
      runFork(Fiber.interrupt(pollingFiber))
      pollingFiber = null
    }
  }

  function onNewTransfers(cb: (newTxs: Array<TransferListItem>) => void) {
    callback = cb
  }

  return { onNewTransfers, destroy }
}
