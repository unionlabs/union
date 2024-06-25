<script lang="ts">
import { Tooltip as TooltipPrimitive } from "bits-ui"
import { cn, flyAndScale } from "$lib/utilities/shadcn.js"

type $$Props = TooltipPrimitive.ContentProps

let className: $$Props["class"] = undefined
export let sideOffset: $$Props["sideOffset"] = 4
export let transition: $$Props["transition"] = flyAndScale
export let transitionConfig: $$Props["transitionConfig"] = {
  y: 8,
  duration: 150
}
export { className as class }
</script>

<TooltipPrimitive.Content
	{transition}
	{transitionConfig}
	{sideOffset}
	class={cn(
		"z-40 overflow-hidden border bg-popover bg-black px-3 py-1.5 text-sm text-popover-foreground shadow-md",
		className
	)}
	{...$$restProps}
>
	<slot />
</TooltipPrimitive.Content>
