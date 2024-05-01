<script lang="ts">
import { Tabs as TabsPrimitive } from "bits-ui"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = TabsPrimitive.TriggerProps
type $$Events = TabsPrimitive.TriggerEvents

let className: $$Props["class"] = undefined
export let value: $$Props["value"]
export { className as class }
</script>

<TabsPrimitive.Trigger
  class={cn(
    'inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm',
    className,
  )}
  {value}
  {...$$restProps}
  on:click
>
  <slot />
</TabsPrimitive.Trigger>
