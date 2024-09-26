<script lang="ts">
import Print from "$lib/components/Terminal/Print.svelte"

type Props = {
  min?: number
  max: number | null
  current: number | null
}

let { min = 1, max, current }: Props = $props()

let progress = $derived(
  current != null && max != null
    ? max === min
      ? 99.99
      : Math.min(((max - current) / (max - min)) * 100, 99.99)
    : 0
)

const TOTAL_SYMBOLS = 30

function generateQueueBar(progress: number): string {
  const filledSymbols = Math.round((progress / 100) * TOTAL_SYMBOLS)
  const emptySymbols = TOTAL_SYMBOLS - filledSymbols

  return `[${"=".repeat(filledSymbols)}${"-".repeat(emptySymbols)}]`
}

let queueBar = $derived(generateQueueBar(progress))

let percentageText = $derived(`${progress.toFixed(2)}%`)
</script>

<Print>{queueBar} - {percentageText}</Print>