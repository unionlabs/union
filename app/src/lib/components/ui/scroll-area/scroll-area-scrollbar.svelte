<script lang="ts">
import { ScrollArea as ScrollAreaPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = ScrollAreaPrimitive.ScrollbarProps & {
  orientation?: "vertical" | "horizontal"
}

	interface Props {
		class?: $$Props["class"];
		orientation?: $$Props["orientation"];
		children?: import('svelte').Snippet;
	}

	let { class: className = undefined, orientation = "vertical", children }: Props = $props();

</script>

<ScrollAreaPrimitive.Scrollbar
	{orientation}
	class={cn(
		"flex touch-none select-none transition-colors",
		orientation === "vertical" && "h-full w-2.5 border-l border-l-transparent p-px",
		orientation === "horizontal" && "h-2.5 w-full border-t border-t-transparent p-px",
		className
	)}
>
	{@render children?.()}
	<ScrollAreaPrimitive.Thumb
		class={cn("bg-border relative rounded-full", orientation === "vertical" && "flex-1")}
	/>
</ScrollAreaPrimitive.Scrollbar>
