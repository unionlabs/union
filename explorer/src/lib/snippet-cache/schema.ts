// Cache schema types - defines valid cache keys and their value types

// Dynamic cache keys follow patterns like:
// - "blocks:recent", "blocks:latest", "blocks:recent:full"
// - "block:{height}"
// - "txs:recent", "txs:height:{height}"
// - "tx:{hash}"
// - "validators:bonded"
// - "validator:{address}", "validator:{address}:delegations"
// - "account:{address}:info", "account:{address}:balances", etc.
// - "chain:stats" - chain stats from indexer (supply, bonded, inflation)
// - "proposals:all", "proposal:{id}", "proposal-tally:{id}", "proposal-votes:{id}"

export type CacheKey = string

export type CacheValue<K extends CacheKey> = unknown

export interface CacheSchema {
  [key: string]: unknown
}
