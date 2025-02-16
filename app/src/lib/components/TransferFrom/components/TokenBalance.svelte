<script lang="ts">
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.js"
import type { Intents } from "$lib/components/TransferFrom/transfer/types.js"
import { formatUnits } from "viem"

interface Props {
  rawIntents: RawIntentsStore
  intents: Intents
}

export let rawIntents: Props["rawIntents"]
export let intents: Props["intents"]

// function calculateMaxAmount() {
//   if (!(intents.baseToken && intents.baseTokenInfo)) return
//   try {
//     const balance = BigInt(intents.baseToken.balance)
//     rawIntents.updateField("amount", formatUnits(balance, intents.baseTokenInfo.combined.decimals))
//   } catch (error) {
//     console.error("Error calculating max amount:", error)
//   }
// }

// $: formattedBalance =
//   intents.baseToken && intents.baseTokenInfo
//     ? formatUnits(BigInt(intents.baseToken.balance), intents.baseTokenInfo.combined.decimals)
//     : "0"
//
// $: isEnabled = !!intents.baseToken && !!intents.baseTokenInfo
// $: hasBalance = isEnabled && BigInt(intents.baseToken?.balance || 0) > 0n
</script>

<div class="text-xs flex justify-between mt-2">
  {JSON.stringify(intents.baseToken)}
  <!-- <span class="text-muted-foreground">Balance: <span class="text-primary">{formattedBalance}</span></span> !-->
  <!-- {#if hasBalance} !-->
  <!--   <button !-->
  <!--           class="ml-1 text-primary font-bold disabled:cursor-not-allowed" !-->
  <!--           on:click={calculateMaxAmount} !-->
  <!--           disabled={!isEnabled} !-->
  <!--   > !-->
  <!--     MAX !-->
  <!--   </button> !-->
  <!-- {/if} !-->
</div>
