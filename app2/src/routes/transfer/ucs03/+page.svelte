<script lang="ts">
import { RawIntentsStoreSvelte } from "../raw-intents-store.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03"
import {
  TransferSubmission,
  nextState,
  hasFailedExit,
  isComplete,
  type TransactionEvmParams
} from "$lib/services/transfer-ucs03-evm"
import { sepolia } from "viem/chains"
import { getAddress } from "viem"
import { bech32AddressToHex } from "@unionlabs/client"
import { Effect } from "effect"
import { generateSalt } from "$lib/services/transfer-ucs03-evm/salt.ts"

export const rawIntents = new RawIntentsStoreSvelte()

/* Hack to be able to JSON.stringify BigInt */
interface BigInt {
  toJSON: () => string
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}

let transferState = $state<TransferSubmission>(TransferSubmission.Filling())

const submit = async () => {
  const receiver = "union10z7xxj2m8q3f7j58uxmff38ws9u8m0vmne2key"
  const formattedReceiver = receiver.startsWith("0x")
    ? getAddress(receiver)
    : getAddress(bech32AddressToHex({ address: receiver }))

  let currentTransactionParams: TransactionEvmParams = {
    chain: sepolia,
    address: "0x84f074c15513f15baea0fbed3ec42f0bd1fb3efa",
    args: {
      sourceChainId: 11155111,
      baseToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
      baseAmount: 100n,
      quoteToken:
        "0x756e696f6e313370786b747532686b387073656b7361616b6135346e677879666d706a6c6a726c65683363633873787671346478616c76747471646d64677635",
      quoteAmount: 100n,
      receiver: formattedReceiver,
      sourceChannelId: 9,
      wethQuoteToken:
        "0x756e696f6e31686373343677677033637775723679336c7a733638706b776765687930636777766e637472747a7932666e3630343772346561717a34646b6c6c",
      timeoutHeight: 0n,
      timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
      salt: Effect.runSync(generateSalt)
    }
  }

  transferState = await nextState(transferState, currentTransactionParams)
  while (!hasFailedExit(transferState)) {
    transferState = await nextState(transferState, currentTransactionParams)
    if (isComplete(transferState)) {
      break
    }
  }
}
</script>

<Sections>
  <SectionTitle>Transfer ucs03 evm</SectionTitle>
  <div class="flex flex-col gap-4">
    <div class="flex gap-4">
      <Button
              class="mt-4 self-start"
              variant="primary"
              onclick={submit}
              disabled={transferState._tag !== "Filling" && !hasFailedExit(transferState) && !isComplete(transferState)}
      >
        {#if transferState._tag !== "Filling" && !hasFailedExit(transferState) && !isComplete(transferState)}
          Submitting...
        {:else if hasFailedExit(transferState)}
          Retry
        {:else}
          Submit
        {/if}
      </Button>
    </div>
    {JSON.stringify(transferState, null, 2)}
  </div>
</Sections>
