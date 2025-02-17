<script lang="ts">
import { deviceWidth } from "$lib/utilities/device.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import LoadingDots from "$lib/components/loading-dots.svelte"
import NumberFlow from "@number-flow/svelte"

export let label: string
export let value: number | string
export let blink: boolean

function formatValue(value: number | string): string {
  if (typeof value === "number") {
    value = value.toLocaleString()
  }
  return value
}

let valueAnimated = 0

$: if (!Number.isNaN(+value)) {
  valueAnimated = +value
}
</script>

<div class={cn("capitalize  flex px-6 py-4 border-none", $deviceWidth >= 888 ? "w-fit" : "w-full")}>
  <div class="text-xl font-bold" on:copy={(event) => {
      event?.clipboardData?.setData('text/plain', value.toString());
      event.preventDefault();
    }}>
    <h3 class="text-xs pt-2 text-muted-foreground">{label}</h3>
    {#if !value}
      <div class="flex justify-start">
        <LoadingDots class="size-6"/>
      </div>
    {:else }
      <p class="text-xl font-bold"><NumberFlow value={valueAnimated}/></p>
    {/if}
  </div>
  <slot/>
</div>
