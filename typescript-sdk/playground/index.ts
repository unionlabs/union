import { http } from "viem"
import { consola } from "scripts/logger.ts"
import { privateKeyToAccount } from "viem/accounts"
import { createMultiUnionClient, type TransferAssetsParameters } from "#mod.ts"

const account = privateKeyToAccount(`0x${process.env["PRIVATE_KEY"]}`)

const clients = createMultiUnionClient([
  {
    chainId: "stride-internal-1",
    transport: http("stride.testnet-1.stridenet.co")
  },
  {
    account,
    chainId: "11155111",
    transport: http("https://rpc.sepolia.org")
  }
])

const payload = {
  amount: 1n,
  autoApprove: false,
  destinationChainId: "stride-internal-1",
  denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789", // LINK
  receiver: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
} satisfies TransferAssetsParameters<"11155111">

const gasResponse = await clients["11155111"].simulateTransaction(payload)

if (gasResponse.isErr()) {
  consola.error(gasResponse.error)
  process.exit(1)
}

consola.success(`gas: ${gasResponse.value}`)

const approvalResponse = await clients["11155111"].approveTransaction(payload)

if (approvalResponse.isErr()) {
  consola.error(approvalResponse.error)
  process.exit(1)
}

consola.box(`Approval success: ${approvalResponse.value}`)

const sepoliaTransfer = await clients["11155111"].transferAsset(payload)

if (sepoliaTransfer.isErr()) {
  console.error(sepoliaTransfer.error)
  process.exit(1)
}

consola.success(`Transfer success: ${sepoliaTransfer.value}`)
