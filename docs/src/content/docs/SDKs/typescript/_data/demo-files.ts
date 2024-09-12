import { dedent } from "ts-dedent"
import type { FileSystemTree } from "@webcontainer/api"

export const demoFiles = {
  "mod.ts": {
    file: {
      contents: dedent(/* ts */ `
import { http } from "viem"
import { arbitrumSepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, type TransferAssetsParameters } from "@unionlabs/client"

const account = privateKeyToAccount(\`0x$\{process.env["PRIVATE_KEY"]\}\`)

console.info(\`Account address: $\{JSON.stringify(account, undefined, 2)\}\`)

const client = createUnionClient({
  account,
  chainId: \`$\{arbitrumSepolia.id\}\`,
  transport: http("https://sepolia-rollup.arbitrum.io/rpc")
})

const payload = {
  amount: 1n,
  autoApprove: false,
  destinationChainId: "stride-internal-1",
  receiver: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3",
  denomAddress: "0xb1d4538b4571d411f07960ef2838ce337fe1e80e" // LINK
} satisfies TransferAssetsParameters<\`$\{typeof arbitrumSepolia.id\}\`>

const gasResponse = await client.simulateTransaction(payload)

if (gasResponse.isErr()) {
  console.error(\`Gas estimation error: $\{gasResponse.error\}\`)
  process.exit(1)
}

console.info(\`Gas estimation: $\{gasResponse.value\}\`)

const approval = await client.approveTransaction(payload)

if (approval.isErr()) {
  console.error(\`Approval error: $\{approval.error\}\`)
  process.exit(1)
}

console.info(\`Approval hash: $\{approval.value\}\`)

const transfer = await client.transferAsset(payload)

if (transfer.isErr()) {
  console.error(\`Transfer error: $\{transfer.error\}\`)
  process.exit(1)
}

console.info(\`Transfer hash: $\{transfer.value\}\`)
        `).trim()
    }
  },
  ".npmrc": {
    file: {
      contents: dedent(`
          engine-strict=true
          npm_config_yes=true
          auto-install-peers=true
          enable-pre-post-scripts=true
          strict-peer-dependencies=false
          node-options="--no-warnings"
        `).trim()
    }
  },
  "package.json": {
    file: {
      contents: dedent(/* json */ `
          {
            "name": "demo",
            "type": "module",
            "dependencies": {
              "tsx": "latest",
              "viem": "latest",
              "@unionlabs/client": "latest"
            }
          }
        `).trim()
    }
  }
} satisfies FileSystemTree
