#!/usr/bin/env node
import nodeUtil from "node:util"

const { values } = nodeUtil.parseArgs({
  args: process.argv.slice(2),
  options: { address: { type: "string" } }
})

const address = values["address"]

console.info(
  nodeUtil.styleText("bgWhiteBright", nodeUtil.styleText("blackBright", `You passed: ${address}`))
)

/**
 * Usage:
 * node cli.js --address "0xf4212614C7Fe0B3feef75057E88b2E77a7E23e83"
 */
