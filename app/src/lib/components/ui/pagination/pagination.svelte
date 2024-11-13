<script lang="ts">
import { Pagination as PaginationPrimitive } from "bits-ui"

import { cn } from "$lib/utilities/shadcn.js"

type $$Props = PaginationPrimitive.Props
type $$Events = PaginationPrimitive.Events

	interface Props {
		class?: $$Props["class"];
		count?: $$Props["count"];
		perPage?: $$Props["perPage"];
		page?: $$Props["page"];
		siblingCount?: $$Props["siblingCount"];
		children?: import('svelte').Snippet<[any]>;
		[key: string]: any
	}

	let {
		class: className = undefined,
		count = 0,
		perPage = 10,
		page = $bindable(1),
		siblingCount = 1,
		children,
		...rest
	}: Props = $props();


let currentPage = $derived(page)

	const children_render = $derived(children);
</script>

<PaginationPrimitive.Root
	{count}
	{perPage}
	{siblingCount}
	bind:page
	
	
	
	asChild
	{...rest}
>
	{#snippet children({ builder, pages, range })}
		<nav {...builder} class={cn("flex w-full flex-col items-center", className)}>
			{@render children_render?.({ pages, range, currentPage, })}
		</nav>
	{/snippet}
</PaginationPrimitive.Root>
