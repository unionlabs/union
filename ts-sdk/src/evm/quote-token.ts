import { Effect } from "effect"
import type { Hex } from "viem"
import { PublicDestinationViemClient } from "./client.js"

export const quoteToken = (baseToken: Hex) =>
  Effect.gen(function* () {
    let client = (yield* PublicDestinationViemClient).client
  })
