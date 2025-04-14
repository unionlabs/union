<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import type { StatisticItem } from "@unionlabs/sdk/schema"
import NumberFlow from "@number-flow/svelte"
import { onMount } from "svelte"

type Props = {
  statistic: StatisticItem
  class?: string
}

const { statistic, class: className = "" }: Props = $props()
let displayValue = $state(1000000)
let isFirstLoad = $state(true)

// Update displayValue whenever statistic.value changes
$effect(() => {
  if (isFirstLoad) {
    // On first load, animate from 0 to the value
    onMount(() => {
      setTimeout(() => {
        displayValue = statistic.value
        isFirstLoad = false
      }, 300) // Small delay to sync with card fade-in
    })
  } else {
    // For subsequent updates, directly update the value
    displayValue = statistic.value
  }
})

// Mapping of statistic names to display names
const displayNames: Record<string, string> = {
  total_packets: "Total Packets",
  total_fungible_asset_orders: "Total Transfers"
}

// Format statistic name for display
function formatStatName(name: string): string {
  // Use the mapping if available
  if (displayNames[name]) {
    return displayNames[name]
  }

  // Otherwise, replace underscores with spaces and capitalize each word
  return name
    .split("_")
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}
</script>

<Card class="h-22 transition-all hover:shadow-lg {className}">
  <Label>{formatStatName(statistic.name)}</Label>
  <p class="text-2xl font-bold mt-1">
    <NumberFlow value={displayValue} transformTiming={{duration:1500, easing: 'ease-out'}} opacityTiming={{duration:1500, easing: 'ease-out'}} spinTiming={{duration:1500, easing: 'ease-out'}} />
  </p>
</Card>
