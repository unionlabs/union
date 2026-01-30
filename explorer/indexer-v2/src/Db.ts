import { Database as BunDB } from "bun:sqlite"
import { Context, Effect, Layer } from "effect"
import { statSync } from "node:fs"
import { IndexerConfigService } from "./config.js"
import { DatabaseError } from "./errors.js"

// ============ Types ============

export interface Block {
  chain_id: string
  height: number
  hash: string
  time: string
  proposer: string
  tx_count: number
  header: unknown
  signatures: unknown
  tx_hashes: string[]
}

export interface Transaction {
  chain_id: string
  hash: string
  height: number
  index: number
  code: number
  codespace: string
  gas_used: string
  gas_wanted: string
  messages: unknown
  memo: string
  fee: unknown
  events: unknown
  raw_log: string
  tx_bytes: string
  timestamp: string
}

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
  insertBlock(block: Block): Effect.Effect<void, DatabaseError>
  insertBlocks(blocks: Block[]): Effect.Effect<void, DatabaseError>
  insertTransaction(tx: Transaction): Effect.Effect<void, DatabaseError>
  insertTransactions(txs: Transaction[]): Effect.Effect<void, DatabaseError>

  // Block queries
  getBlocks(chainId: string, limit: number, before?: number): Effect.Effect<Block[], DatabaseError>
  getBlockByHeight(chainId: string, height: number): Effect.Effect<Block | null, DatabaseError>
  getBlockByHash(chainId: string, hash: string): Effect.Effect<Block | null, DatabaseError>
  getLatestBlock(chainId: string): Effect.Effect<Block | null, DatabaseError>
  getLatestHeight(chainId: string): Effect.Effect<number, DatabaseError>
  getMinHeight(chainId: string): Effect.Effect<number, DatabaseError>
  getBlockCount(chainId: string): Effect.Effect<number, DatabaseError>

  // Transaction queries
  getTransactions(
    chainId: string,
    limit: number,
    before?: number,
  ): Effect.Effect<Transaction[], DatabaseError>
  getTransactionByHash(
    chainId: string,
    hash: string,
  ): Effect.Effect<Transaction | null, DatabaseError>
  getTransactionsByHeight(
    chainId: string,
    height: number,
  ): Effect.Effect<Transaction[], DatabaseError>
  getTransactionsByAddress(
    chainId: string,
    address: string,
    limit: number,
  ): Effect.Effect<Transaction[], DatabaseError>
  getTxCount(chainId: string): Effect.Effect<number, DatabaseError>

  // Analytics
  getMsgTypeStats(chainId: string): Effect.Effect<MsgTypeStats[], DatabaseError>
  getDailyStats(chainId: string, days: number): Effect.Effect<DailyStats[], DatabaseError>
  getHourlyStats(chainId: string, hours: number): Effect.Effect<HourlyStats[], DatabaseError>

  // Chain stats
  insertChainStats(stats: ChainStats): Effect.Effect<void, DatabaseError>
  getLatestChainStats(chainId: string): Effect.Effect<ChainStats | null, DatabaseError>
  getChainStatsHistory(chainId: string, limit: number): Effect.Effect<ChainStats[], DatabaseError>

  // Maintenance
  pruneOldData(chainId: string, keepHeight: number): Effect.Effect<void, DatabaseError>

  // Stats
  getDbSizeBytes(): Effect.Effect<number, DatabaseError>

  // Proxy cache
  getCached(key: string): Effect.Effect<string | null, DatabaseError>
  setCache(key: string, value: string, ttlSeconds: number): Effect.Effect<void, DatabaseError>
  pruneExpiredCache(): Effect.Effect<number, DatabaseError>
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

  // Prune in a transaction for atomicity
  const pruneTransaction = db.transaction((chainId: string, keepHeight: number) => {
    db.run(`DELETE FROM blocks WHERE chain_id = ? AND height < ?`, [chainId, keepHeight])
    db.run(`DELETE FROM transactions WHERE chain_id = ? AND height < ?`, [chainId, keepHeight])
    db.run(`DELETE FROM chain_stats WHERE chain_id = ? AND height < ?`, [chainId, keepHeight])
  })

  // Safe JSON parse with fallback
  const safeJsonParse = <T>(value: unknown, fallback: T): T => {
    if (typeof value !== "string" || !value) {
      return fallback
    }
    try {
      return JSON.parse(value) as T
    } catch {
      return fallback
    }
  }

  // Parse block row with validation
  const parseBlock = (row: Record<string, unknown>): Block => ({
    chain_id: String(row.chain_id ?? ""),
    height: Number(row.height ?? 0),
    hash: String(row.hash ?? ""),
    time: String(row.time ?? ""),
    proposer: String(row.proposer ?? ""),
    tx_count: Number(row.tx_count ?? 0),
    header: safeJsonParse(row.header, null),
    signatures: safeJsonParse(row.signatures, []),
    tx_hashes: safeJsonParse(row.tx_hashes, []),
  })

  // Parse transaction row with validation
  const parseTx = (row: Record<string, unknown>): Transaction => ({
    chain_id: String(row.chain_id ?? ""),
    hash: String(row.hash ?? ""),
    height: Number(row.height ?? 0),
    index: Number(row.tx_index ?? 0),
    code: Number(row.code ?? 0),
    codespace: String(row.codespace ?? ""),
    gas_used: String(row.gas_used ?? "0"),
    gas_wanted: String(row.gas_wanted ?? "0"),
    messages: safeJsonParse(row.messages, []),
    memo: String(row.memo ?? ""),
    fee: safeJsonParse(row.fee, {}),
    events: safeJsonParse(row.events, []),
    raw_log: String(row.raw_log ?? ""),
    tx_bytes: String(row.tx_bytes ?? ""),
    timestamp: String(row.timestamp ?? ""),
  })

  // Wrap database operations with proper error handling
  const dbEffect = <T>(operation: string, fn: () => T): Effect.Effect<T, DatabaseError> =>
    Effect.try({
      try: fn,
      catch: (error) =>
        new DatabaseError({
          operation,
          message: error instanceof Error ? error.message : String(error),
        }),
    })

  return {
    // Inserts
    insertBlock: (block) =>
      dbEffect("insertBlock", () => {
        insertBlocksBatch([block])
      }),

    insertBlocks: (blocks) =>
      dbEffect("insertBlocks", () => {
        if (blocks.length > 0) {
          insertBlocksBatch(blocks)
        }
      }),

    insertTransaction: (tx) =>
      dbEffect("insertTransaction", () => {
        insertTxsBatch([tx])
      }),

    insertTransactions: (txs) =>
      dbEffect("insertTransactions", () => {
        if (txs.length > 0) {
          insertTxsBatch(txs)
        }
      }),

    // Block queries
    getBlocks: (chainId, limit, before) =>
      dbEffect("getBlocks", () => {
        if (before !== undefined) {
          const rows = db
            .query<Record<string, unknown>, [string, number, number]>(
              `SELECT * FROM blocks WHERE chain_id = ? AND height < ? ORDER BY height DESC LIMIT ?`,
            )
            .all(chainId, before, limit)
          return rows.map(parseBlock)
        }
        const rows = db
          .query<Record<string, unknown>, [string, number]>(
            `SELECT * FROM blocks WHERE chain_id = ? ORDER BY height DESC LIMIT ?`,
          )
          .all(chainId, limit)
        return rows.map(parseBlock)
      }),

    getBlockByHeight: (chainId, height) =>
      dbEffect("getBlockByHeight", () => {
        const row = db
          .query<Record<string, unknown>, [string, number]>(
            `SELECT * FROM blocks WHERE chain_id = ? AND height = ?`,
          )
          .get(chainId, height)
        return row ? parseBlock(row) : null
      }),

    getBlockByHash: (chainId, hash) =>
      dbEffect("getBlockByHash", () => {
        const row = db
          .query<Record<string, unknown>, [string, string]>(
            `SELECT * FROM blocks WHERE chain_id = ? AND hash = ?`,
          )
          .get(chainId, hash)
        return row ? parseBlock(row) : null
      }),

    getLatestBlock: (chainId) =>
      dbEffect("getLatestBlock", () => {
        const row = db
          .query<Record<string, unknown>, [string]>(
            `SELECT * FROM blocks WHERE chain_id = ? ORDER BY height DESC LIMIT 1`,
          )
          .get(chainId)
        return row ? parseBlock(row) : null
      }),

    getLatestHeight: (chainId) =>
      dbEffect("getLatestHeight", () => {
        const row = db
          .query<{ height: number | null }, [string]>(
            `SELECT MAX(height) as height FROM blocks WHERE chain_id = ?`,
          )
          .get(chainId)
        return row?.height ?? 0
      }),

    getMinHeight: (chainId) =>
      dbEffect("getMinHeight", () => {
        const row = db
          .query<{ height: number | null }, [string]>(
            `SELECT MIN(height) as height FROM blocks WHERE chain_id = ?`,
          )
          .get(chainId)
        return row?.height ?? 0
      }),

    getBlockCount: (chainId) =>
      dbEffect("getBlockCount", () => {
        const row = db
          .query<{ count: number }, [string]>(
            `SELECT COUNT(*) as count FROM blocks WHERE chain_id = ?`,
          )
          .get(chainId)
        return row?.count ?? 0
      }),

    // Transaction queries
    getTransactions: (chainId, limit, before) =>
      dbEffect("getTransactions", () => {
        if (before !== undefined) {
          const rows = db
            .query<Record<string, unknown>, [string, number, number]>(
              `SELECT * FROM transactions WHERE chain_id = ? AND height < ? ORDER BY height DESC, tx_index ASC LIMIT ?`,
            )
            .all(chainId, before, limit)
          return rows.map(parseTx)
        }
        const rows = db
          .query<Record<string, unknown>, [string, number]>(
            `SELECT * FROM transactions WHERE chain_id = ? ORDER BY height DESC, tx_index ASC LIMIT ?`,
          )
          .all(chainId, limit)
        return rows.map(parseTx)
      }),

    getTransactionByHash: (chainId, hash) =>
      dbEffect("getTransactionByHash", () => {
        const row = db
          .query<Record<string, unknown>, [string, string]>(
            `SELECT * FROM transactions WHERE chain_id = ? AND hash = ?`,
          )
          .get(chainId, hash)
        return row ? parseTx(row) : null
      }),

    getTransactionsByHeight: (chainId, height) =>
      dbEffect("getTransactionsByHeight", () => {
        const rows = db
          .query<Record<string, unknown>, [string, number]>(
            `SELECT * FROM transactions WHERE chain_id = ? AND height = ? ORDER BY tx_index ASC`,
          )
          .all(chainId, height)
        return rows.map(parseTx)
      }),

    getTransactionsByAddress: (chainId, address, limit) =>
      dbEffect("getTransactionsByAddress", () => {
        // Use parameterized LIKE patterns for safety
        const senderPattern = `%"sender":"${address.replace(/[%_\\]/g, "\\$&")}"%`
        const receiverPattern = `%"receiver":"${address.replace(/[%_\\]/g, "\\$&")}"%`
        const rows = db
          .query<Record<string, unknown>, [string, string, string, number]>(
            `SELECT * FROM transactions
             WHERE chain_id = ? AND (messages LIKE ? ESCAPE '\\' OR messages LIKE ? ESCAPE '\\')
             ORDER BY height DESC LIMIT ?`,
          )
          .all(chainId, senderPattern, receiverPattern, limit)
        return rows.map(parseTx)
      }),

    getTxCount: (chainId) =>
      dbEffect("getTxCount", () => {
        const row = db
          .query<{ count: number }, [string]>(
            `SELECT COUNT(*) as count FROM transactions WHERE chain_id = ?`,
          )
          .get(chainId)
        return row?.count ?? 0
      }),

    // Analytics
    getMsgTypeStats: (chainId) =>
      dbEffect("getMsgTypeStats", () =>
        db
          .query<MsgTypeStats, [string]>(
            `
          SELECT
            json_extract(messages, '$[0]."@type"') as msg_type,
            COUNT(*) as count,
            SUM(CASE WHEN code = 0 THEN 1 ELSE 0 END) as success_count,
            SUM(CASE WHEN code != 0 THEN 1 ELSE 0 END) as failure_count
          FROM transactions WHERE chain_id = ?
          GROUP BY json_extract(messages, '$[0]."@type"')
          ORDER BY count DESC
        `,
          )
          .all(chainId)),

    getDailyStats: (chainId, days) =>
      dbEffect("getDailyStats", () =>
        db
          .query<DailyStats, [string, string]>(
            `
          SELECT
            date(time) as date,
            COUNT(*) as block_count,
            SUM(tx_count) as tx_count
          FROM blocks
          WHERE chain_id = ? AND time >= datetime('now', ?)
          GROUP BY date(time)
          ORDER BY date DESC
        `,
          )
          .all(chainId, `-${days} days`)),

    getHourlyStats: (chainId, hours) =>
      dbEffect("getHourlyStats", () =>
        db
          .query<HourlyStats, [string, string]>(
            `
          SELECT
            strftime('%Y-%m-%d %H:00', timestamp) as hour,
            COUNT(*) as count
          FROM transactions
          WHERE chain_id = ? AND timestamp >= datetime('now', ?)
          GROUP BY strftime('%Y-%m-%d %H:00', timestamp)
          ORDER BY hour DESC
        `,
          )
          .all(chainId, `-${hours} hours`)),

    // Chain stats
    insertChainStats: (stats) =>
      dbEffect("insertChainStats", () => {
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
      dbEffect("getLatestChainStats", () => {
        const row = db
          .query<ChainStats, [string]>(
            `SELECT * FROM chain_stats WHERE chain_id = ? ORDER BY height DESC LIMIT 1`,
          )
          .get(chainId)
        return row ?? null
      }),

    getChainStatsHistory: (chainId, limit) =>
      dbEffect("getChainStatsHistory", () =>
        db
          .query<ChainStats, [string, number]>(
            `SELECT * FROM chain_stats WHERE chain_id = ? ORDER BY height DESC LIMIT ?`,
          )
          .all(chainId, limit)),

    // Maintenance - atomic transaction for all deletions
    pruneOldData: (chainId, keepHeight) =>
      dbEffect("pruneOldData", () => {
        pruneTransaction(chainId, keepHeight)
      }),

    // Stats
    getDbSizeBytes: () =>
      dbEffect("getDbSizeBytes", () => {
        let totalSize = 0
        try {
          totalSize += statSync(dbPath).size
        } catch {
          /* file may not exist */
        }
        try {
          totalSize += statSync(`${dbPath}-wal`).size
        } catch {
          /* WAL may not exist */
        }
        try {
          totalSize += statSync(`${dbPath}-shm`).size
        } catch {
          /* SHM may not exist */
        }
        return totalSize
      }),

    // Proxy cache - get with lazy expiry check
    getCached: (key) =>
      dbEffect("getCached", () => {
        const now = Math.floor(Date.now() / 1000)
        const row = db
          .query<{ value: string; expires_at: number }, [string]>(
            `SELECT value, expires_at FROM proxy_cache WHERE key = ?`,
          )
          .get(key)
        if (!row) {
          return null
        }
        if (row.expires_at < now) {
          // Expired - delete and return null
          db.run(`DELETE FROM proxy_cache WHERE key = ?`, [key])
          return null
        }
        return row.value
      }),

    // Proxy cache - set with TTL
    setCache: (key, value, ttlSeconds) =>
      dbEffect("setCache", () => {
        const now = Math.floor(Date.now() / 1000)
        db.run(
          `INSERT OR REPLACE INTO proxy_cache (key, value, cached_at, expires_at) VALUES (?, ?, ?, ?)`,
          [key, value, now, now + ttlSeconds],
        )
      }),

    // Prune expired cache entries - returns count deleted
    pruneExpiredCache: () =>
      dbEffect("pruneExpiredCache", () => {
        const now = Math.floor(Date.now() / 1000)
        const result = db.run(`DELETE FROM proxy_cache WHERE expires_at < ?`, [now])
        return result.changes
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
      header TEXT,
      signatures TEXT,
      tx_hashes TEXT,
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
      messages TEXT,
      memo TEXT,
      fee TEXT,
      events TEXT,
      raw_log TEXT,
      tx_bytes TEXT,
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

    -- Proxy response cache (for upstream requests)
    CREATE TABLE IF NOT EXISTS proxy_cache (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL,
      cached_at INTEGER NOT NULL,
      expires_at INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_cache_expires ON proxy_cache(expires_at);
  `)
}

// ============ Layer ============

export const DatabaseLive = Layer.scoped(
  Database,
  Effect.gen(function*() {
    const config = yield* IndexerConfigService

    const dbPath = config.dbPath
    const db = new BunDB(dbPath, { create: true })

    // Proper cleanup on shutdown
    yield* Effect.addFinalizer(() =>
      Effect.sync(() => {
        db.close()
      }).pipe(Effect.tap(() => Effect.log("Database closed")))
    )

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
