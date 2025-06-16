import type { Database as BetterSqlite3Database } from "better-sqlite3"
import { Effect } from "effect"

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
  
export function getAggregateIncident(db: BetterSqlite3Database, key: string): string | undefined {
  const row = db
    .prepare(`SELECT incident_id FROM aggregate_incidents WHERE key = ?`)
    .get(key) as { incident_id: string } | undefined
  return row?.incident_id
}

export function markAggregateIncident(db: BetterSqlite3Database, key: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO aggregate_incidents
      (key, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(key, incidentId)
}

export function clearAggregateIncident(db: BetterSqlite3Database, key: string) {
  db.prepare(`DELETE FROM aggregate_incidents WHERE key = ?`).run(key)
}

export function getSupplyIncident(db: BetterSqlite3Database, key: string): string | undefined {
  const row = db.prepare(`SELECT incident_id FROM supply_incidents WHERE key = ?`).get(key) as
    | { incident_id: string }
    | undefined
  return row?.incident_id
}

export function markSupplyIncident(db: BetterSqlite3Database, key: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO supply_incidents
      (key, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(key, incidentId)
}

export function clearSupplyIncident(db: BetterSqlite3Database, key: string) {
  db.prepare(`DELETE FROM supply_incidents WHERE key = ?`).run(key)
}

export function getPendingSupply(
  db: BetterSqlite3Database,
  key: string,
): boolean {
  const row = db
    .prepare(`SELECT 1 FROM pending_supply_mismatch WHERE key = ?`)
    .get(key)
  return !!row
}

export function markPendingSupply(
  db: BetterSqlite3Database,
  key: string,
) {
  db
    .prepare(`
      INSERT OR REPLACE INTO pending_supply_mismatch
        (key, inserted_at)
      VALUES (?, strftime('%s','now')*1000)
    `)
    .run(key)
}

export function clearPendingSupply(
  db: BetterSqlite3Database,
  key: string,
) {
  db.prepare(`DELETE FROM pending_supply_mismatch WHERE key = ?`).run(key)
}

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
export function getSslIncident(db: BetterSqlite3Database, url: string): string | undefined {
  const row = db.prepare(`SELECT incident_id FROM ssl_incidents WHERE url = ?`).get(url) as
    | { incident_id: string }
    | undefined

  return row?.incident_id?.length ? row.incident_id : undefined
}

export function markSslIncident(db: BetterSqlite3Database, url: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO ssl_incidents
      (url, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(url, incidentId)
}

export function clearSslIncident(db: BetterSqlite3Database, url: string) {
  db.prepare(`DELETE FROM ssl_incidents WHERE url = ?`).run(url)
}

export function addFunded(db: BetterSqlite3Database, txHash: string) {
  db.prepare(`INSERT OR IGNORE INTO funded_txs (transaction_hash) VALUES (?)`).run(txHash)
}


export const dbPrepeare = (db: BetterSqlite3Database) =>
    Effect.gen(function*() {
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

        db.prepare(`
        CREATE TABLE IF NOT EXISTS ssl_incidents (
            url          TEXT    PRIMARY KEY,
            incident_id  TEXT    NOT NULL,
            inserted_at  INTEGER
        )
        `).run()

        db.prepare(`
        CREATE TABLE IF NOT EXISTS supply_incidents (
            key            TEXT PRIMARY KEY,  -- e.g. "$srcChain:dstChain:token.denom"
            incident_id    TEXT NOT NULL,
            inserted_at    INTEGER
        )
        `).run()

        db.prepare(`
        CREATE TABLE IF NOT EXISTS pending_supply_mismatch (
            key         TEXT PRIMARY KEY,
            inserted_at INTEGER
        )
        `).run()

        db.prepare(`
        CREATE TABLE IF NOT EXISTS aggregate_incidents (
            key            TEXT PRIMARY KEY,  -- e.g. "chainId:tokenAddr"
            incident_id    TEXT NOT NULL,
            inserted_at    INTEGER
        )
        `).run()    
    })