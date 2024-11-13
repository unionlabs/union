<script lang="ts">
import { Pagination as PaginationPrimitive } from "bits-ui"
import ChevronRight from "virtual:icons/lucide/chevron-right"
import { Button } from "$lib/components/ui/button/index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = PaginationPrimitive.NextButtonProps
type $$Events = PaginationPrimitive.NextButtonEvents

	interface Props {
		class?: $$Props["class"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, children, ...rest }: Props = $props();


	const children_render = $derived(children);
</script>

<PaginationPrimitive.NextButton asChild >
	{#snippet children({ builder })}
		<Button
			variant="ghost"
			class={cn("gap-1 pr-2.5", className)}
			builders={[builder]}
			on:click
			{...rest}
		>
			{#if children_render}{@render children_render()}{:else}
				<span>Next</span>
				<ChevronRight class="h-4 w-4" />
			{/if}
		</Button>
	{/snippet}
</PaginationPrimitive.NextButton>
