<script lang="ts">
import { deviceWidth } from "$lib/utilities/device.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import LoadingDots from "$lib/components/loading-dots.svelte"

  interface Props {
    label: string;
    value: number | string;
    blink: boolean;
    children?: import('svelte').Snippet;
  }

  let {
    label,
    value,
    blink,
    children
  }: Props = $props();

function formatValue(value: number | string): string {
  if (typeof value === "number") {
    value = value.toLocaleString()
  }
  return value
}
</script>

<div class={cn("uppercase  flex px-6 py-4 border-none", $deviceWidth >= 888 ? "w-fit" : "w-full")}>
  <div class="text-xl pt-2 font-bold divide-y" oncopy={(event) => {
      event?.clipboardData?.setData('text/plain', value.toString());
      event.preventDefault();
    }}>
    <h3 class="text-sm pb-2">{label}</h3>
    {#if !value}
      <div class="flex justify-start">
        <LoadingDots class="size-6"/>
      </div>
    {:else }
      {#key value}
        <p class="text-xl pt-2 font-bold" class:blink={blink}>{formatValue(value)}</p>
      {/key}
    {/if}
  </div>
  {@render children?.()}
</div>

<style>
    .blink {
        animation: blink-animation 0.5s;
    }

    @keyframes blink-animation {
        0% { @apply text-accent; }
        100% { @apply text-primary; }
    }
</style>

