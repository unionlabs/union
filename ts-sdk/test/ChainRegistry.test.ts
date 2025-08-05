import { assert, describe, it } from "@effect/vitest"
import * as ChainRegistry from "@unionlabs/sdk/ChainRegistry"
import * as GraphQL from "@unionlabs/sdk/GraphQL"
import { Chain } from "@unionlabs/sdk/schema/chain"
import { Effect } from "effect"
import { UniversalChainId } from "../src/schema/chain.js"
import { IN_NIX_BUILD } from "./utils.js"

describe.skipIf(IN_NIX_BUILD)("ChainRegistry", () => {
  it.layer(GraphQL.GraphQL.Default)(
    "GraphQL.Default",
    (it) =>
      it.effect("get by id", () =>
        Effect.gen(function*() {
          assert.instanceOf(
            yield* ChainRegistry.getChainById(
              UniversalChainId.make("ethereum.11155111"),
            ),
            Chain,
          )
        })),
  )
})
