<script lang="ts">
import * as ResizablePrimitive from "paneforge"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = ResizablePrimitive.PaneGroupProps

let className: $$Props["class"] = undefined
export let direction: $$Props["direction"]
export let paneGroup: $$Props["paneGroup"] = undefined
export let el: $$Props["el"] = undefined
export { className as class }
</script>

<ResizablePrimitive.PaneGroup
	bind:el
	bind:paneGroup
	{direction}
	class={cn("flex h-full w-full data-[direction=vertical]:flex-col", className)}
	{...$$restProps}
>
	<slot />
</ResizablePrimitive.PaneGroup>
