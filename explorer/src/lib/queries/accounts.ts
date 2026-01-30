import { CosmosClient } from "$lib/services/cosmos-client"
import { Effect } from "effect"

export const fetchAccount = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getAccount(address)
  })

export const fetchBalances = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getBalances(address)
  })

export const fetchDelegations = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getDelegations(address)
  })

export const fetchUnbondingDelegations = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getUnbondingDelegations(address)
  })

export const fetchDelegatorRewards = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getDelegatorTotalRewards(address)
  })

export const fetchAccountTxs = (address: string, page = 1, limit = 20) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.searchTxs(`message.sender='${address}'`, page, limit)
  })

export const fetchReceivedTxs = (address: string, page = 1, limit = 20) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    // Use transfer.recipient for incoming transfers
    return yield* client.searchTxs(`transfer.recipient='${address}'`, page, limit)
  })

// Fetch all transactions related to an address from multiple event types
export const fetchAllAccountTxs = (address: string, page = 1, limit = 25) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient

    // Query multiple event types in parallel
    const [senderTxs, coinReceivedTxs, coinSpentTxs] = yield* Effect.all([
      client.searchTxs(`message.sender='${address}'`, page, limit),
      client.searchTxs(`coin_received.receiver='${address}'`, page, limit),
      client.searchTxs(`coin_spent.spender='${address}'`, page, limit),
    ], { concurrency: 3 })

    // Merge and deduplicate by txhash
    const txMap = new Map<string, typeof senderTxs.tx_responses[0]>()

    for (const tx of senderTxs.tx_responses) {
      txMap.set(tx.txhash, tx)
    }
    for (const tx of coinReceivedTxs.tx_responses) {
      if (!txMap.has(tx.txhash)) {
        txMap.set(tx.txhash, tx)
      }
    }
    for (const tx of coinSpentTxs.tx_responses) {
      if (!txMap.has(tx.txhash)) {
        txMap.set(tx.txhash, tx)
      }
    }

    // Sort by height descending (most recent first)
    const allTxs = Array.from(txMap.values()).sort((a, b) =>
      parseInt(b.height) - parseInt(a.height)
    )

    return {
      tx_responses: allTxs,
      pagination: {
        next_key: null,
        total: String(allTxs.length),
      },
      // Track if any source might have more
      hasMore: senderTxs.tx_responses.length >= limit
        || coinReceivedTxs.tx_responses.length >= limit
        || coinSpentTxs.tx_responses.length >= limit,
    }
  })
