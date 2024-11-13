<script lang="ts">
import { AlertDialog as AlertDialogPrimitive } from "bits-ui"
import { buttonVariants } from "$lib/components/ui/button/index.js"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = AlertDialogPrimitive.ActionProps
type $$Events = AlertDialogPrimitive.ActionEvents

	interface Props {
		class?: $$Props["class"];
		children?: import('svelte').Snippet<[any]>;
		[key: string]: any
	}

	let { class: className = undefined, children, ...rest }: Props = $props();


	const children_render = $derived(children);
</script>

<AlertDialogPrimitive.Action
	class={cn(buttonVariants(), className)}
	{...rest}
	on:click
	on:keydown
	
>
	{#snippet children({ builder })}
		{@render children_render?.({ builder, })}
	{/snippet}
</AlertDialogPrimitive.Action>
