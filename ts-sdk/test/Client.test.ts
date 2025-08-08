import { describe, it } from "@effect/vitest"
import { Client, ClientRequest } from "@unionlabs/sdk"
import { Effect, pipe } from "effect"

describe("Client", () => {
  it.effect("scratch", () =>
    Effect.gen(function*() {
      const client = yield* Client.Client.pipe()

      const transfer = pipe(
        ClientRequest.send("0x123", "bbn1abc", {
          batch: true,
        }),
        ClientRequest.withTokenBy(x => x.name === "WBTC"),
      )

      const result = yield* client.execute(transfer)
    }))
})
