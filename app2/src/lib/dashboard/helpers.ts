import type { User } from "@supabase/supabase-js"
import { Effect, Option, pipe } from "effect"
import { AuthenticationError } from "./errors"
import type { AuthProvider } from "./stores/user.svelte"

export const hasProviderLinked = (user: User, provider: AuthProvider) =>
  user.identities?.some(identity => identity.provider === provider) ?? false

export const getProviderId = (user: User, provider: AuthProvider) =>
  pipe(
    user.identities,
    Option.fromNullable,
    Option.flatMap(identities =>
      Option.fromNullable(
        identities.find(id => id.provider.toLowerCase() === provider.toLowerCase()),
      )
    ),
    Option.map(identity => identity.id),
  )

export const isProviderConnected = (user: User, provider: AuthProvider) =>
  pipe(
    user.identities,
    Option.fromNullable,
    Option.map(identities =>
      identities.some(id => id.provider.toLowerCase() === provider.toLowerCase())
    ),
    Option.getOrElse(() => false),
  )

export const requireAuthenticatedUserId = (
  user: unknown,
): Effect.Effect<unknown, AuthenticationError, string> => {
  return pipe(
    Option.fromNullable((user as any)?.session?.user?.id),
    Option.match({
      onNone: () =>
        Effect.fail(
          new AuthenticationError({ cause: "User is not authenticated" }),
        ),
      onSome: (userId) => Effect.succeed(userId),
    }),
  )
}
