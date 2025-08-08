import { assert, describe, it } from "@effect/vitest"
import { ZkgmClient } from "@unionlabs/sdk"
import { EvmZkgmClient } from "@unionlabs/sdk-evm"
import { Effect, Layer } from "effect"

describe("BrowserHttpClient", () => {
  it.effect("json", () =>
    Effect.gen(function*() {
      const client = yield* ZkgmClient.ZkgmClient
    }))
})
