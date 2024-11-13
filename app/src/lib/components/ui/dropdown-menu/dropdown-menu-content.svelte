<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui"
import { cn, flyAndScale } from "$lib/utilities/shadcn.js"

type $$Props = DropdownMenuPrimitive.ContentProps
type $$Events = DropdownMenuPrimitive.ContentEvents

	interface Props {
		class?: $$Props["class"];
		sideOffset?: $$Props["sideOffset"];
		transition?: $$Props["transition"];
		transitionConfig?: $$Props["transitionConfig"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let {
		class: className = undefined,
		sideOffset = 4,
		transition = flyAndScale,
		transitionConfig = undefined,
		children,
		...rest
	}: Props = $props();

</script>

<DropdownMenuPrimitive.Content
	{transition}
	{transitionConfig}
	{sideOffset}
	class={cn(
		"z-50 min-w-[8rem] rounded-md border bg-popover p-1 text-popover-foreground shadow-md focus:outline-none",
		className
	)}
	{...rest}
	on:keydown
>
	{@render children?.()}
</DropdownMenuPrimitive.Content>
