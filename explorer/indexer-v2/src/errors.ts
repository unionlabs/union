import { Schema } from "effect"

// ============ Domain Error Types ============

/** Database operation failed */
export class DatabaseError extends Schema.TaggedError<DatabaseError>()("DatabaseError", {
  operation: Schema.String,
  message: Schema.String,
}) {}

/** Upstream RPC/REST call failed */
export class UpstreamError extends Schema.TaggedError<UpstreamError>()("UpstreamError", {
  endpoint: Schema.String,
  message: Schema.String,
}) {}

// ============ Error Union ============

export type IndexerError = DatabaseError | UpstreamError
