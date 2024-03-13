<script lang="ts">
import { Tabs as TabsPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.ts"

type $$Props = TabsPrimitive.TriggerProps
type $$Events = TabsPrimitive.TriggerEvents

let className: $$Props["class"]
export let value: $$Props["value"]
export { className as class }
</script>

<TabsPrimitive.Trigger
  class={cn(
    'inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
    className
  )}
  {value}
  {...$$restProps}
  on:click
  on:keydown
  on:focus
>
  <slot />
</TabsPrimitive.Trigger>
