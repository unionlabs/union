import { browser } from "$app/environment"

// Indexer URL - serves REST/RPC proxy with CORS enabled
// Uses VITE_ prefix for client-side access, falls back to process.env for server-side
export const INDEXER_URL = browser
  ? (import.meta.env.VITE_INDEXER_URL || "http://localhost:3002")
  : (process.env.INDEXER_URL || import.meta.env.VITE_INDEXER_URL || "http://localhost:3002")
