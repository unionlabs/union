import { fallback, http } from "viem"
import { createUnionClient } from "#mod.js"

/**
 * evm chain, where ERC20 approval is a thing
 */
createUnionClient({
  chainId: "11155111",
  transport: fallback([http("https://rpc.sepolia.org")])
}).approveTransaction

/**
 * cosmos sdk chain, where there's no concept of token approval
 */
createUnionClient({
  chainId: "stride-internal-1",
  transport: http("stride.testnet-1.stridenet.co")
}).approveTransaction
