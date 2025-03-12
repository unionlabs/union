<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import type { StatisticItem } from "$lib/schema/statistics"

type Props = {
  statistic: StatisticItem
  class?: string
}

const { statistic, class: className = "" }: Props = $props()

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

// Format large numbers with commas
function formatNumber(num: number): string {
  return num.toLocaleString()
}
</script>

<Card class="h-22 transition-all hover:shadow-lg {className}">
  <Label>{formatStatName(statistic.name)}</Label>
  <p class="text-2xl font-bold mt-1">{formatNumber(statistic.value)}</p>
</Card>
