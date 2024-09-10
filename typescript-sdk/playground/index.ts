import { http } from "viem"
import { consola } from "scripts/logger.ts"
import { privateKeyToAccount } from "viem/accounts"
import { createMultiUnionClient } from "../dist/index.mjs"
import type { TransferAssetsParameters } from "../dist/index.d.ts"

const account = privateKeyToAccount(`0x${process.env["PRIVATE_KEY"]}`)

const clients = createMultiUnionClient([
  {
    chainId: "stride-internal-1",
    transport: http("stride.testnet-1.stridenet.co")
  },
  {
    account,
    chainId: "421614",
    transport: http("https://sepolia-rollup.arbitrum.io/rpc")
  }
])

const payload = {
  amount: 1n,
  autoApprove: true,
  destinationChainId: "stride-internal-1",
  denomAddress: "0xb1d4538b4571d411f07960ef2838ce337fe1e80e", // LINK
  receiver: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
} satisfies TransferAssetsParameters<"421614">

const gasResponse = await clients["421614"].simulateTransaction(payload)

if (gasResponse.isErr()) {
  consola.error(gasResponse.error)
  process.exit(1)
}

consola.success(`gas: ${gasResponse.value}`)

// const sepoliaTransfer = await clients['421614'].transferAsset(payload)

// if (sepoliaTransfer.isErr()) {
//   console.error(sepoliaTransfer.error)
//   process.exit(1)
// }

// consola.success(`Transfer success: ${sepoliaTransfer.value}`)
