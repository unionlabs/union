import { Database as BunDB } from "bun:sqlite"
import { Context, Effect, Layer } from "effect"
import { statSync } from "node:fs"
import { IndexerConfigService } from "./config.js"

// ============ Types ============

export interface Block {
  chain_id: string
  height: number
  hash: string
  time: string
  proposer: string
  tx_count: number
  // Full data
  header: unknown // Full block header JSON
  signatures: unknown // Commit signatures JSON
  tx_hashes: string[] // List of tx hashes in this block
}

export interface Transaction {
  chain_id: string
  hash: string
  height: number
  index: number // Position in block
  code: number
  codespace: string
  gas_used: string
  gas_wanted: string
  // Full data
  messages: unknown // Full messages array JSON
  memo: string
  fee: unknown // Fee JSON {amount, gas_limit, payer, granter}
  events: unknown // Full events array JSON
  raw_log: string
  tx_bytes: string // Base64 encoded raw tx
  timestamp: string
}

// Chain stats (supply, staking)
export interface ChainStats {
  chain_id: string
  height: number
  timestamp: string
  total_supply: string
  bonded_tokens: string
  not_bonded_tokens: string
  inflation: string
  community_pool: string
}

// Analytics types
export interface MsgTypeStats {
  msg_type: string
  count: number
  success_count: number
  failure_count: number
}

export interface DailyStats {
  date: string
  block_count: number
  tx_count: number
}

export interface HourlyStats {
  hour: string
  count: number
}

// ============ Database Service ============

export interface Db {
  // Inserts
  insertBlock(block: Block): Effect.Effect<void>
  insertBlocks(blocks: Block[]): Effect.Effect<void>
  insertTransaction(tx: Transaction): Effect.Effect<void>
  insertTransactions(txs: Transaction[]): Effect.Effect<void>

  // Block queries
  getBlocks(chainId: string, limit: number, before?: number): Effect.Effect<Block[]>
  getBlockByHeight(chainId: string, height: number): Effect.Effect<Block | null>
  getBlockByHash(chainId: string, hash: string): Effect.Effect<Block | null>
  getLatestBlock(chainId: string): Effect.Effect<Block | null>
  getLatestHeight(chainId: string): Effect.Effect<number>
  getMinHeight(chainId: string): Effect.Effect<number>
  getBlockCount(chainId: string): Effect.Effect<number>

  // Transaction queries
  getTransactions(chainId: string, limit: number, before?: number): Effect.Effect<Transaction[]>
  getTransactionByHash(chainId: string, hash: string): Effect.Effect<Transaction | null>
  getTransactionsByHeight(chainId: string, height: number): Effect.Effect<Transaction[]>
  getTransactionsByAddress(
    chainId: string,
    address: string,
    limit: number,
  ): Effect.Effect<Transaction[]>
  getTxCount(chainId: string): Effect.Effect<number>

  // Analytics
  getMsgTypeStats(chainId: string): Effect.Effect<MsgTypeStats[]>
  getDailyStats(chainId: string, days: number): Effect.Effect<DailyStats[]>
  getHourlyStats(chainId: string, hours: number): Effect.Effect<HourlyStats[]>

  // Chain stats
  insertChainStats(stats: ChainStats): Effect.Effect<void>
  getLatestChainStats(chainId: string): Effect.Effect<ChainStats | null>
  getChainStatsHistory(chainId: string, limit: number): Effect.Effect<ChainStats[]>

  // Maintenance
  pruneOldData(chainId: string, keepHeight: number): Effect.Effect<void>

  // Stats
  getDbSizeBytes(): Effect.Effect<number>
}

export class Database extends Context.Tag("Database")<Database, Db>() {}

// ============ Implementation ============

function createDb(db: BunDB, dbPath: string): Db {
  // Prepared statements for inserts
  const insertBlockStmt = db.prepare(`
    INSERT OR REPLACE INTO blocks
    (chain_id, height, hash, time, proposer, tx_count, header, signatures, tx_hashes)
    VALUES ($chain_id, $height, $hash, $time, $proposer, $tx_count, $header, $signatures, $tx_hashes)
  `)

  const insertTxStmt = db.prepare(`
    INSERT OR REPLACE INTO transactions
    (chain_id, hash, height, tx_index, code, codespace, gas_used, gas_wanted, messages, memo, fee, events, raw_log, tx_bytes, timestamp)
    VALUES ($chain_id, $hash, $height, $tx_index, $code, $codespace, $gas_used, $gas_wanted, $messages, $memo, $fee, $events, $raw_log, $tx_bytes, $timestamp)
  `)

  const insertBlocksBatch = db.transaction((blocks: Block[]) => {
    for (const b of blocks) {
      insertBlockStmt.run({
        $chain_id: b.chain_id,
        $height: b.height,
        $hash: b.hash,
        $time: b.time,
        $proposer: b.proposer,
        $tx_count: b.tx_count,
        $header: JSON.stringify(b.header),
        $signatures: JSON.stringify(b.signatures),
        $tx_hashes: JSON.stringify(b.tx_hashes),
      })
    }
  })

  const insertTxsBatch = db.transaction((txs: Transaction[]) => {
    for (const t of txs) {
      insertTxStmt.run({
        $chain_id: t.chain_id,
        $hash: t.hash,
        $height: t.height,
        $tx_index: t.index,
        $code: t.code,
        $codespace: t.codespace || "",
        $gas_used: t.gas_used,
        $gas_wanted: t.gas_wanted,
        $messages: JSON.stringify(t.messages),
        $memo: t.memo || "",
        $fee: JSON.stringify(t.fee),
        $events: JSON.stringify(t.events),
        $raw_log: t.raw_log || "",
        $tx_bytes: t.tx_bytes || "",
        $timestamp: t.timestamp,
      })
    }
  })

  // Helper to parse JSON fields from DB
  const parseBlock = (row: Record<string, unknown>): Block => ({
    chain_id: row.chain_id as string,
    height: row.height as number,
    hash: row.hash as string,
    time: row.time as string,
    proposer: row.proposer as string,
    tx_count: row.tx_count as number,
    header: JSON.parse(row.header as string || "null"),
    signatures: JSON.parse(row.signatures as string || "[]"),
    tx_hashes: JSON.parse(row.tx_hashes as string || "[]"),
  })

  const parseTx = (row: Record<string, unknown>): Transaction => ({
    chain_id: row.chain_id as string,
    hash: row.hash as string,
    height: row.height as number,
    index: row.tx_index as number,
    code: row.code as number,
    codespace: row.codespace as string,
    gas_used: row.gas_used as string,
    gas_wanted: row.gas_wanted as string,
    messages: JSON.parse(row.messages as string || "[]"),
    memo: row.memo as string,
    fee: JSON.parse(row.fee as string || "{}"),
    events: JSON.parse(row.events as string || "[]"),
    raw_log: row.raw_log as string,
    tx_bytes: row.tx_bytes as string,
    timestamp: row.timestamp as string,
  })

  return {
    // Inserts
    insertBlock: (block) => Effect.sync(() => insertBlocksBatch([block])),

    insertBlocks: (blocks) =>
      Effect.sync(() => {
        if (blocks.length > 0) {
          insertBlocksBatch(blocks)
        }
      }),

    insertTransaction: (tx) => Effect.sync(() => insertTxsBatch([tx])),

    insertTransactions: (txs) =>
      Effect.sync(() => {
        if (txs.length > 0) {
          insertTxsBatch(txs)
        }
      }),

    // Block queries
    getBlocks: (chainId, limit, before) =>
      Effect.sync(() => {
        if (before !== undefined) {
          const rows = db.query<Record<string, unknown>, [string, number, number]>(
            `SELECT * FROM blocks WHERE chain_id = ? AND height < ? ORDER BY height DESC LIMIT ?`,
          ).all(chainId, before, limit)
          return rows.map(parseBlock)
        }
        const rows = db.query<Record<string, unknown>, [string, number]>(
          `SELECT * FROM blocks WHERE chain_id = ? ORDER BY height DESC LIMIT ?`,
        ).all(chainId, limit)
        return rows.map(parseBlock)
      }),

    getBlockByHeight: (chainId, height) =>
      Effect.sync(() => {
        const row = db.query<Record<string, unknown>, [string, number]>(
          `SELECT * FROM blocks WHERE chain_id = ? AND height = ?`,
        ).get(chainId, height)
        return row ? parseBlock(row) : null
      }),

    getBlockByHash: (chainId, hash) =>
      Effect.sync(() => {
        const row = db.query<Record<string, unknown>, [string, string]>(
          `SELECT * FROM blocks WHERE chain_id = ? AND hash = ?`,
        ).get(chainId, hash)
        return row ? parseBlock(row) : null
      }),

    getLatestBlock: (chainId) =>
      Effect.sync(() => {
        const row = db.query<Record<string, unknown>, [string]>(
          `SELECT * FROM blocks WHERE chain_id = ? ORDER BY height DESC LIMIT 1`,
        ).get(chainId)
        return row ? parseBlock(row) : null
      }),

    getLatestHeight: (chainId) =>
      Effect.sync(() => {
        const row = db.query<{ height: number | null }, [string]>(
          `SELECT MAX(height) as height FROM blocks WHERE chain_id = ?`,
        ).get(chainId)
        return row?.height || 0
      }),

    getMinHeight: (chainId) =>
      Effect.sync(() => {
        const row = db.query<{ height: number | null }, [string]>(
          `SELECT MIN(height) as height FROM blocks WHERE chain_id = ?`,
        ).get(chainId)
        return row?.height || 0
      }),

    getBlockCount: (chainId) =>
      Effect.sync(() => {
        const row = db.query<{ count: number }, [string]>(
          `SELECT COUNT(*) as count FROM blocks WHERE chain_id = ?`,
        ).get(chainId)
        return row?.count || 0
      }),

    // Transaction queries
    getTransactions: (chainId, limit, before) =>
      Effect.sync(() => {
        if (before !== undefined) {
          const rows = db.query<Record<string, unknown>, [string, number, number]>(
            `SELECT * FROM transactions WHERE chain_id = ? AND height < ? ORDER BY height DESC, tx_index ASC LIMIT ?`,
          ).all(chainId, before, limit)
          return rows.map(parseTx)
        }
        const rows = db.query<Record<string, unknown>, [string, number]>(
          `SELECT * FROM transactions WHERE chain_id = ? ORDER BY height DESC, tx_index ASC LIMIT ?`,
        ).all(chainId, limit)
        return rows.map(parseTx)
      }),

    getTransactionByHash: (chainId, hash) =>
      Effect.sync(() => {
        const row = db.query<Record<string, unknown>, [string, string]>(
          `SELECT * FROM transactions WHERE chain_id = ? AND hash = ?`,
        ).get(chainId, hash)
        return row ? parseTx(row) : null
      }),

    getTransactionsByHeight: (chainId, height) =>
      Effect.sync(() => {
        const rows = db.query<Record<string, unknown>, [string, number]>(
          `SELECT * FROM transactions WHERE chain_id = ? AND height = ? ORDER BY tx_index ASC`,
        ).all(chainId, height)
        return rows.map(parseTx)
      }),

    getTransactionsByAddress: (chainId, address, limit) =>
      Effect.sync(() => {
        // Escape LIKE wildcards in address to prevent injection
        const escaped = address.replace(/[%_]/g, "\\$&")
        const rows = db.query<Record<string, unknown>, [string, string, string, number]>(
          `SELECT * FROM transactions
           WHERE chain_id = ? AND (messages LIKE ? ESCAPE '\\' OR messages LIKE ? ESCAPE '\\')
           ORDER BY height DESC LIMIT ?`,
        ).all(chainId, `%"sender":"${escaped}"%`, `%"receiver":"${escaped}"%`, limit)
        return rows.map(parseTx)
      }),

    getTxCount: (chainId) =>
      Effect.sync(() => {
        const row = db.query<{ count: number }, [string]>(
          `SELECT COUNT(*) as count FROM transactions WHERE chain_id = ?`,
        ).get(chainId)
        return row?.count || 0
      }),

    // Analytics
    getMsgTypeStats: (chainId) =>
      Effect.sync(() => {
        // Extract first message type from messages JSON array
        return db.query<MsgTypeStats, [string]>(`
          SELECT
            json_extract(messages, '$[0]."@type"') as msg_type,
            COUNT(*) as count,
            SUM(CASE WHEN code = 0 THEN 1 ELSE 0 END) as success_count,
            SUM(CASE WHEN code != 0 THEN 1 ELSE 0 END) as failure_count
          FROM transactions WHERE chain_id = ?
          GROUP BY json_extract(messages, '$[0]."@type"')
          ORDER BY count DESC
        `).all(chainId)
      }),

    getDailyStats: (chainId, days) =>
      Effect.sync(() =>
        db.query<DailyStats, [string, string]>(`
          SELECT
            date(time) as date,
            COUNT(*) as block_count,
            SUM(tx_count) as tx_count
          FROM blocks
          WHERE chain_id = ? AND time >= datetime('now', ?)
          GROUP BY date(time)
          ORDER BY date DESC
        `).all(chainId, `-${days} days`)
      ),

    getHourlyStats: (chainId, hours) =>
      Effect.sync(() =>
        db.query<HourlyStats, [string, string]>(`
          SELECT
            strftime('%Y-%m-%d %H:00', timestamp) as hour,
            COUNT(*) as count
          FROM transactions
          WHERE chain_id = ? AND timestamp >= datetime('now', ?)
          GROUP BY strftime('%Y-%m-%d %H:00', timestamp)
          ORDER BY hour DESC
        `).all(chainId, `-${hours} hours`)
      ),

    // Chain stats
    insertChainStats: (stats) =>
      Effect.sync(() => {
        db.run(
          `
          INSERT OR REPLACE INTO chain_stats
          (chain_id, height, timestamp, total_supply, bonded_tokens, not_bonded_tokens, inflation, community_pool)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        `,
          [
            stats.chain_id,
            stats.height,
            stats.timestamp,
            stats.total_supply,
            stats.bonded_tokens,
            stats.not_bonded_tokens,
            stats.inflation,
            stats.community_pool,
          ],
        )
      }),

    getLatestChainStats: (chainId) =>
      Effect.sync(() => {
        const row = db.query<ChainStats, [string]>(
          `SELECT * FROM chain_stats WHERE chain_id = ? ORDER BY height DESC LIMIT 1`,
        ).get(chainId)
        return row || null
      }),

    getChainStatsHistory: (chainId, limit) =>
      Effect.sync(() => {
        return db.query<ChainStats, [string, number]>(
          `SELECT * FROM chain_stats WHERE chain_id = ? ORDER BY height DESC LIMIT ?`,
        ).all(chainId, limit)
      }),

    // Maintenance
    pruneOldData: (chainId, keepHeight) =>
      Effect.sync(() => {
        db.run(`DELETE FROM blocks WHERE chain_id = ? AND height < ?`, [chainId, keepHeight])
        db.run(`DELETE FROM transactions WHERE chain_id = ? AND height < ?`, [chainId, keepHeight])
        db.run(`DELETE FROM chain_stats WHERE chain_id = ? AND height < ?`, [chainId, keepHeight])
      }),

    // Stats
    getDbSizeBytes: () =>
      Effect.sync(() => {
        let totalSize = 0
        // Main database file
        try {
          totalSize += statSync(dbPath).size
        } catch { /* file may not exist */ }
        // WAL file
        try {
          totalSize += statSync(`${dbPath}-wal`).size
        } catch { /* WAL may not exist */ }
        // SHM file
        try {
          totalSize += statSync(`${dbPath}-shm`).size
        } catch { /* SHM may not exist */ }
        return totalSize
      }),
  }
}

// ============ Schema ============

function initSchema(db: BunDB): void {
  db.exec(`
    -- Blocks with full data
    CREATE TABLE IF NOT EXISTS blocks (
      chain_id TEXT NOT NULL,
      height INTEGER NOT NULL,
      hash TEXT NOT NULL,
      time TEXT NOT NULL,
      proposer TEXT NOT NULL,
      tx_count INTEGER NOT NULL,
      header TEXT,          -- Full block header JSON
      signatures TEXT,      -- Commit signatures JSON array
      tx_hashes TEXT,       -- Array of tx hashes in this block
      PRIMARY KEY (chain_id, height)
    );

    -- Transactions with full data
    CREATE TABLE IF NOT EXISTS transactions (
      chain_id TEXT NOT NULL,
      hash TEXT NOT NULL,
      height INTEGER NOT NULL,
      tx_index INTEGER NOT NULL,
      code INTEGER NOT NULL,
      codespace TEXT,
      gas_used TEXT NOT NULL,
      gas_wanted TEXT NOT NULL,
      messages TEXT,        -- Full messages array JSON
      memo TEXT,
      fee TEXT,             -- Fee JSON object
      events TEXT,          -- Full events array JSON
      raw_log TEXT,
      tx_bytes TEXT,        -- Base64 encoded raw tx
      timestamp TEXT NOT NULL,
      PRIMARY KEY (chain_id, hash)
    );

    -- Chain stats (supply, staking, etc)
    CREATE TABLE IF NOT EXISTS chain_stats (
      chain_id TEXT NOT NULL,
      height INTEGER NOT NULL,
      timestamp TEXT NOT NULL,
      total_supply TEXT NOT NULL,
      bonded_tokens TEXT NOT NULL,
      not_bonded_tokens TEXT NOT NULL,
      inflation TEXT,
      community_pool TEXT,
      PRIMARY KEY (chain_id, height)
    );

    -- Indexes for common queries
    CREATE INDEX IF NOT EXISTS idx_blocks_chain_height ON blocks(chain_id, height DESC);
    CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(chain_id, hash);
    CREATE INDEX IF NOT EXISTS idx_txs_chain_height ON transactions(chain_id, height DESC);
    CREATE INDEX IF NOT EXISTS idx_txs_hash ON transactions(hash);
    CREATE INDEX IF NOT EXISTS idx_txs_timestamp ON transactions(timestamp);
    CREATE INDEX IF NOT EXISTS idx_chain_stats_chain_height ON chain_stats(chain_id, height DESC);
  `)
}

// ============ Layer ============

export const DatabaseLive = Layer.effect(
  Database,
  Effect.gen(function*() {
    const config = yield* IndexerConfigService

    const dbPath = config.dbPath.replace(".duckdb", ".sqlite")
    const db = new BunDB(dbPath, { create: true })

    // Optimize for write-heavy workload
    db.exec("PRAGMA journal_mode = WAL")
    db.exec("PRAGMA synchronous = NORMAL")
    db.exec("PRAGMA cache_size = -128000") // 128MB cache
    db.exec("PRAGMA temp_store = MEMORY")
    db.exec("PRAGMA mmap_size = 1073741824") // 1GB mmap

    initSchema(db)
    yield* Effect.log(`Database initialized: ${dbPath}`)

    return createDb(db, dbPath)
  }),
)
