import type { Database } from "$lib/dashboard/database.types.ts"
import { createClient } from "@supabase/supabase-js"
import { Effect } from "effect"
import { SupabaseClientError } from "./errors"

export type Entity<T extends keyof (Database["public"]["Tables"] & Database["public"]["Views"])> =
  (Database["public"]["Tables"] & Database["public"]["Views"])[T]["Row"]

let client: ReturnType<typeof createClient<Database>> | null = null

export const getSupabaseClient = () =>
  Effect.gen(function*() {
    if (client) {
      return client
    }

    const url = import.meta.env.VITE_SUPABASE_URL
    const anonKey = import.meta.env.VITE_SUPABASE_ANON_KEY

    if (!url || !anonKey) {
      return yield* Effect.fail(
        new SupabaseClientError({
          cause: "Missing Supabase URL or anonymous key",
          operation: "init",
        }),
      )
    }

    client = createClient<Database>(url, anonKey, {
      auth: {
        autoRefreshToken: true,
      },
    })

    return client
  })
