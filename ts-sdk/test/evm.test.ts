import { describe, it, expect } from "vitest"
import { ucs03abi } from "../src/evm/abi/index.js"

describe("abi", () => {
  it("should export ucs03abi", () => {
    expect(ucs03abi).toBeDefined()
    expect(Array.isArray(ucs03abi)).toBe(true)
    expect(ucs03abi.length).toBeGreaterThan(0)

    // Check that it has the expected functions
    const functionNames = ucs03abi.map(item => item.name)
    expect(functionNames).toContain("transferV2")
    expect(functionNames).toContain("transferAndCall")
  })
})
