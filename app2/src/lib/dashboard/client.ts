import { PUBLIC_SUPABASE_ANON_KEY, PUBLIC_SUPABASE_URL } from "$env/static/public"
import type { Database } from "$lib/dashboard/database.types.ts"
import { createClient } from "@supabase/supabase-js"
import { Effect, Schema as S } from "effect"
import { SupabaseClientError } from "./errors"

export type Entity<T extends keyof (Database["public"]["Tables"] & Database["public"]["Views"])> =
  (Database["public"]["Tables"] & Database["public"]["Views"])[T]["Row"]

export type SupabaseOptions = NonNullable<Parameters<typeof createClient<Database>>[2]>

export class SupabaseClient extends Effect.Service<SupabaseClient>()("SupabaseClient", {
  scoped: (options?: SupabaseOptions | undefined) =>
    Effect.gen(function*() {
      const url = yield* S.decode(S.URL)(PUBLIC_SUPABASE_URL).pipe(
        Effect.mapError((cause) =>
          new SupabaseClientError({
            operation: "init",
            message: "Could not decode PUBLIC_SUPABASE_URL to URL",
            cause,
          })
        ),
      )
      const anonKey = yield* S.decode(S.NonEmptyString)(PUBLIC_SUPABASE_ANON_KEY).pipe(
        Effect.mapError((cause) =>
          new SupabaseClientError({
            operation: "init",
            message: "Could not decode PUBLIC_SUPABASE_ANON_KEY to non-empty string",
            cause,
          })
        ),
      )

      return createClient<Database>(url.toString(), anonKey, {
        ...options,
        auth: {
          ...options?.auth,
          autoRefreshToken: true,
        },
      })
    }),
}) {}
