<script lang="ts">
import InlineLoadingDots from "$lib/components/InlineLoadingDots.svelte"
import LoadingDots from "$lib/components/loading-dots.svelte"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.js"
import type { Intents } from "$lib/components/TransferFrom/transfer/types.js"
import { formatUnits } from "viem"

interface Props {
  rawIntents: RawIntentsStore
  intents: Intents
}

export let rawIntents: Props["rawIntents"]
export let intents: Props["intents"]

function calculateMaxAmount() {
  if (
    !(intents.baseToken && intents.baseToken?.balance?.kind === "balance" && intents.baseTokenInfo)
  )
    return
  try {
    const balance = BigInt(intents.baseToken.balance.amount || 0)
    rawIntents.updateField("amount", formatUnits(balance, intents.baseTokenInfo.combined.decimals))
  } catch (error) {
    console.error("Error calculating max amount:", error)
  }
}

$: formattedBalance =
  intents.baseToken && intents.baseToken?.balance?.kind === "balance" && intents.baseTokenInfo
    ? formatUnits(
        BigInt(intents.baseToken.balance.amount || 0),
        intents.baseTokenInfo.combined.decimals
      )
    : "0"

$: isEnabled = !!intents.baseToken && !!intents.baseTokenInfo
$: hasBalance =
  isEnabled &&
  intents.baseToken?.balance?.kind === "balance" &&
  BigInt(intents.baseToken?.balance.amount || 0) > 0n
</script>

<div class="text-xs flex justify-between mt-2">
  {#if intents.baseToken}
    {#if intents.baseToken.balance?.kind === "loading"}
      <InlineLoadingDots>Loading Balance</InlineLoadingDots>
    {:else}
    <span class="text-muted-foreground">Balance: <span class="text-primary">{formattedBalance}</span></span>
    {#if hasBalance}
      <button
              class="ml-1 text-primary font-bold disabled:cursor-not-allowed"
              on:click={calculateMaxAmount}
              disabled={!isEnabled}
      >
        MAX
      </button>
      {/if}
    {/if}
  {/if}
</div>
