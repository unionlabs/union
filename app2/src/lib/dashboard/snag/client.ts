import SnagSolutions from "@snagsolutions/sdk"
import { Effect, Option } from "effect"
import { SnagClientError } from "./errors.ts"

let client: Option.Option<SnagSolutions> = Option.none()

export const getSnagClient = () =>
  Effect.gen(function*() {
    return yield* Option.match(client, {
      onNone: () =>
        Effect.gen(function*() {
          const apiKey = import.meta.env.VITE_SNAG_API_KEY

          if (!apiKey) {
            return yield* Effect.fail(
              new SnagClientError({
                cause: "Missing Snag API key",
                operation: "init",
              }),
            )
          }

          const newClient = new SnagSolutions({
            apiKey,
            maxRetries: 2,
            timeout: 20 * 1000,
          })

          client = Option.some(newClient)
          return newClient
        }),
      onSome: (existingClient) => Effect.succeed(existingClient),
    })
  })

export type { SnagSolutions }
