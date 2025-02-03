<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"

export let data: Array<{ count: number; day: string }> = []

$: minValue = 0
$: maxValue = Array.isArray(data) && data.length > 0 ? Math.max(...data.map(d => d.count)) : 0

function normalize(
  value: number,
  min: number,
  max: number,
  newMin: number,
  newMax: number
): number {
  if (min === max) return newMin
  if (Number.isNaN(value)) return newMin
  return Math.max(
    newMin,
    Math.min(newMax, ((value - min) / (max - min)) * (newMax - newMin) + newMin)
  )
}

$: normalizedData =
  Array.isArray(data) && data.length > 0
    ? data.map(d => ({
        ...d,
        normalizedValue: Math.max(0, Math.floor(normalize(d.count, minValue, maxValue, 0, 9)))
      }))
    : []
</script>

<div class="flex flex-row-reverse items-end gap-[2.5px]">
  {#each normalizedData as data}
    {#if Array.isArray(data.normalizedValue)}
    <Tooltip.Root>
      <Tooltip.Trigger>
        <div class="bar flex flex-col-reverse gap-[1px] group cursor-crosshair">
          <div class="half-square bg-primary group-hover:bg-accent h-[2.5px] w-[5px] transition"></div>
          {#if data.normalizedValue > 0}
            {#each Array.from({ length: Math.min(data.normalizedValue, 9) }) as _}
              <div class="square bg-primary group-hover:bg-accent h-[5px] w-[5px] transition"></div>
            {/each}
          {/if}
        </div>
      </Tooltip.Trigger>
      <Tooltip.Content>
        {#if data.day && data.count !== undefined && data.count !== null}
          {new Date(data.day).toISOString().slice(0, 10)}
          <br/>
          {data.count}
        {/if}
      </Tooltip.Content>
    </Tooltip.Root>
    {/if}
  {/each}
</div>