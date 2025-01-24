<script lang="ts">
import type { ValidationStore } from "$lib/components/TransferFrom/transfer/validation.ts"
import { derived, get, type Readable, writable, type Writable } from "svelte/store"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { Button } from "$lib/components/ui/button"
import {
  type EvmChainId,
  createUnionClient,
  evmChainFromChainId,
  type TransferAssetsParameters,
  truncateAddress,
  type AptosBrowserWallet,
  type ChainId,
  http,
  type CosmosChainId
} from "@unionlabs/client"
import { truncate } from "$lib/utilities/format.ts"
import { custom, getConnectorClient, switchChain, waitForTransactionReceipt } from "@wagmi/core"
import { getAddress, type HttpTransport, parseUnits } from "viem"
import { config, userAddrEvm } from "$lib/wallet/evm/config.ts"
import { toast } from "svelte-sonner"
import { aptosStore, getAptosWallet, userAddressAptos } from "$lib/wallet/aptos"
import { stepAfter, stepBefore, type TransferState } from "$lib/transfer/transfer.ts"
import { cosmosStore, getCosmosOfflineSigner, userAddrCosmos } from "$lib/wallet/cosmos"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { raise, sleep } from "$lib/utilities"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import { toIsoString } from "$lib/utilities/date.ts"
import { goto } from "$app/navigation"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import Stepper from "$lib/components/stepper.svelte"
import type { Step } from "$lib/stepper-types.ts"
import Truncate from "$lib/components/truncate.svelte"
import type { Chain, Ucs03Channel } from "$lib/types"

export let chains: Array<Chain>
export let channel: Ucs03Channel
export let transferArgs: {
  baseToken: string
  baseAmount: bigint
  quoteToken: string
  quoteAmount: bigint
  receiver: string
  sourceChannelId: number
  ucs03address: string
}

const REDIRECT_DELAY_MS = 5000
let transferState: Writable<TransferState> = writable({ kind: "PRE_TRANSFER" })

function transfer() {
  const sourceChain = chains.find(c => c.chain_id === sourceChain)
  if (!sourceChain) {
    toast.error("source chain not found")
    return
  }

  if (sourceChain.rpc_type === "evm") {
    evmTransfer(sourceChain)
    return
  }
  toast.error("cosmos currently unsupported")
}

async function evmTransfer(sourceChain: Chain) {
  const connectorClient = await getConnectorClient(config)
  const selectedChain = evmChainFromChainId(sourceChain.chain_id)

  const evmClient = createUnionClient({
    account: connectorClient.account,
    chainId: sourceChain.chain_id as EvmChainId,
    transport: custom(window.ethereum)
  })

  const approveResponse = await evmClient.approveErc20(transferArgs)

  if (approveResponse.isErr()) {
    toast.error(approveResponse.error)
    process.exit(1)
  }
}
</script>

<div class="h-full w-full flex flex-col justify-between p-4 overflow-y-scroll">
  {JSON.stringify(channel)}
  {JSON.stringify(transferArgs)}
<Button
    disabled={!transferArgs}
    on:click={() => transfer()}>Transfer
</Button>
</div>




