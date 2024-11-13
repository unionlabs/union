<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"

  interface Props {
    data?: Array<{ count: number; day: Date }>;
  }

  let { data = [] }: Props = $props();

let minValue = $derived(0)
let maxValue = $derived(data.length > 0 ? Math.max(...data.map(d => d.count)) : 0)

function normalize(
  value: number,
  min: number,
  max: number,
  newMin: number,
  newMax: number
): number {
  if (min === max) return newMin
  return ((value - min) / (max - min)) * (newMax - newMin) + newMin
}

let normalizedData =
  $derived(data.length > 0
    ? data.map(d => ({
        ...d,
        normalizedValue: Math.floor(normalize(d.count, minValue, maxValue, 0, 9))
      }))
    : [])
</script>

<div class="flex flex-row-reverse items-end gap-[2.5px]">
  {#each normalizedData as data}
    <Tooltip.Root>
      <Tooltip.Trigger>
        <div class="bar flex flex-col-reverse gap-[1px] group cursor-crosshair">
          <div class="half-square bg-primary group-hover:bg-accent h-[2.5px] w-[5px] transition"></div>
          {#each Array(data.normalizedValue) as _}
            <div class="square bg-primary group-hover:bg-accent h-[5px] w-[5px] transition"></div>
          {/each}
        </div>
      </Tooltip.Trigger>
      <Tooltip.Content>
        {#if data.day && data.count}
          {new Date(data.day).toISOString().slice(0, 10)}
          <br/>
          {data.count}
        {/if}
      </Tooltip.Content>
    </Tooltip.Root>
  {/each}
</div>

