<script lang="ts">
import GripVertical from "lucide-svelte/icons/grip-vertical"
import * as ResizablePrimitive from "paneforge"
import type { WithoutChildrenOrChild } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  withHandle = false,
  ...restProps
}: WithoutChildrenOrChild<ResizablePrimitive.PaneResizerProps> & {
  withHandle?: boolean
} = $props()
</script>

<ResizablePrimitive.PaneResizer
	bind:ref
	class={cn(
		"bg-border focus-visible:ring-ring relative flex w-px items-center justify-center after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-offset-1 data-[direction=vertical]:h-px data-[direction=vertical]:w-full data-[direction=vertical]:after:left-0 data-[direction=vertical]:after:h-1 data-[direction=vertical]:after:w-full data-[direction=vertical]:after:-translate-y-1/2 data-[direction=vertical]:after:translate-x-0 [&[data-direction=vertical]>div]:rotate-90",
		className
	)}
	{...restProps}
>
	{#if withHandle}
		<div class="bg-border z-10 flex h-4 w-3 items-center justify-center rounded-sm border">
			<GripVertical class="size-2.5" />
		</div>
	{/if}
</ResizablePrimitive.PaneResizer>
