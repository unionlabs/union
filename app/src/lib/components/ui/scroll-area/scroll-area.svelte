<script lang="ts">
import { ScrollArea as ScrollAreaPrimitive } from "bits-ui"
import { Scrollbar } from "./index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = ScrollAreaPrimitive.Props & {
  orientation?: "vertical" | "horizontal" | "both"
  scrollbarXClasses?: string
  scrollbarYClasses?: string
}


	interface Props {
		class?: $$Props["class"];
		orientation?: string;
		scrollbarXClasses?: string;
		scrollbarYClasses?: string;
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let {
		class: className = undefined,
		orientation = "vertical",
		scrollbarXClasses = "",
		scrollbarYClasses = "",
		children,
		...rest
	}: Props = $props();
</script>

<ScrollAreaPrimitive.Root {...rest} class={cn("relative overflow-hidden", className)}>
	<ScrollAreaPrimitive.Viewport class="h-full w-full rounded-[inherit]">
		<ScrollAreaPrimitive.Content>
			{@render children?.()}
		</ScrollAreaPrimitive.Content>
	</ScrollAreaPrimitive.Viewport>
	{#if orientation === "vertical" || orientation === "both"}
		<Scrollbar orientation="vertical" class={scrollbarYClasses} />
	{/if}
	{#if orientation === "horizontal" || orientation === "both"}
		<Scrollbar orientation="horizontal" class={scrollbarXClasses} />
	{/if}
	<ScrollAreaPrimitive.Corner />
</ScrollAreaPrimitive.Root>
