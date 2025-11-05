import { assert, describe, it } from "vitest"
import * as index from "./index.js"

describe("index", () => {
  it.each([
    ["Snippets", index.Snippets],
    ["Runtime", index.Runtime],
    ["SvelteConfigProvider", index.SvelteConfigProvider],
  ])("exports %s", (_, a) => assert.isDefined(a))
})
