import { describe, it, expect } from "vitest"
import { ucs03abi, instructionAbi } from "../src/evm/abi/index.js"

describe("abi", () => {
  it("should export ucs03abi", () => {
    expect(ucs03abi).toBeDefined()
    expect(Array.isArray(ucs03abi)).toBe(true)
    expect(ucs03abi.length).toBeGreaterThan(0)

    // Check that it has the expected functions
    const functionNames = ucs03abi.filter(item => item.type === "function").map(item => item.name)
    expect(functionNames).toContain("send")
    expect(functionNames).toContain("ensureExported")
  })

  it("should export instructionAbi", () => {
    expect(instructionAbi).toBeDefined()
    expect(Array.isArray(instructionAbi)).toBe(true)
    expect(instructionAbi.length).toBeGreaterThan(0)

    const argNames = instructionAbi.map(item => item.name)
    expect(argNames).toContain("operand")
  })
})
