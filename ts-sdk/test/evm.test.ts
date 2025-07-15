import { Ucs03 } from "@unionlabs/sdk"
import { describe, expect, it } from "vitest"

describe("abi", () => {
  it("should export ucs03abi", () => {
    expect(Ucs03.Abi).toBeDefined()
    expect(Array.isArray(Ucs03.Abi)).toBe(true)
    expect(Ucs03.Abi.length).toBeGreaterThan(0)

    const functionNames = Ucs03.Abi.filter(item => item.type === "function").map(item => item.name)
    expect(functionNames).toContain("send")
    expect(functionNames).toContain("ensureExported")
  })

  it("should export instructionAbi", () => {
    expect(Ucs03.InstructionAbi()).toBeDefined()
    expect(Array.isArray(Ucs03.InstructionAbi())).toBe(true)
    expect(Ucs03.InstructionAbi().length).toBeGreaterThan(0)

    const argNames = Ucs03.InstructionAbi().map(item => item.name)
    expect(argNames).toContain("operand")
  })
})
